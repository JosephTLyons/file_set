use std::convert::TryFrom;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;

mod orderable_set;
use orderable_set::OrderableSet;

pub enum Filter {
    Item(ItemFilter),
    // Size(SizeFilter),
    Visibility(VisibilityFilter),
}

pub enum ItemFilter {
    Directory,
    File,
    Symlink,
}

pub enum SizeFilter {
    Bytes,
    Kilobytes,
    Megatbytes,
    Gigabytes,
    Terabytes,
}

pub enum VisibilityFilter {
    Hidden,
    Visible,
}

pub enum OrderBy {
    // Just sort all of them ascending, then the reverse can be applied
    Extension,
    Name,
    Size,
}

pub struct FileSet {
    orderable_set: OrderableSet<PathBuf>,
}

impl FileSet {
    pub fn new(directory: &Path) -> FileSet {
        let mut orderable_set: OrderableSet<PathBuf> = OrderableSet::new();

        for directory_entry in read_dir(&directory).unwrap() {
            orderable_set.push(directory_entry.unwrap().path()).unwrap();
        }

        FileSet { orderable_set }
    }

    pub fn exclude(&mut self, filter: Filter) -> FileSet {
        let items_to_exclude: FileSet = self.filter(filter);
        FileSet {
            orderable_set: self
                .orderable_set
                .difference(&items_to_exclude.orderable_set),
        }
    }

    pub fn filter(&mut self, filter: Filter) -> FileSet {
        FileSet {
            orderable_set: match filter {
                Filter::Item(item) => self.filter_by_item(item),
                Filter::Visibility(visibility) => self.filter_by_visibility(visibility),
            },
        }
    }

    fn filter_by_item(&mut self, item_filter: ItemFilter) -> OrderableSet<PathBuf> {
        let filtered_path_vec: Vec<PathBuf> = match item_filter {
            ItemFilter::Directory => self
                .orderable_set
                .to_vec()
                .into_iter()
                .filter(|x| x.symlink_metadata().unwrap().file_type().is_dir())
                .collect(),
            ItemFilter::File => self
                .orderable_set
                .to_vec()
                .into_iter()
                .filter(|x| x.symlink_metadata().unwrap().file_type().is_file())
                .collect(),
            ItemFilter::Symlink => self
                .orderable_set
                .to_vec()
                .into_iter()
                .filter(|x| x.symlink_metadata().unwrap().file_type().is_symlink())
                .collect(),
        };

        OrderableSet::try_from(filtered_path_vec).unwrap()
    }

    pub fn filter_by_visibility(
        &mut self,
        visibility_filter: VisibilityFilter,
    ) -> OrderableSet<PathBuf> {
        let should_find_visible_files: bool = match visibility_filter {
            VisibilityFilter::Hidden => false,
            VisibilityFilter::Visible => true,
        };

        let filtered_path_vec: Vec<PathBuf> = self
            .orderable_set
            .to_vec()
            .into_iter()
            .filter(|x| {
                should_find_visible_files
                    != x.file_name().unwrap().to_string_lossy().starts_with('.')
            })
            .collect();

        OrderableSet::try_from(filtered_path_vec).unwrap()
    }

    pub fn reverse(&mut self) -> FileSet {
        let mut orderable_set = self.orderable_set.clone();
        orderable_set.reverse();
        FileSet { orderable_set }
    }

    pub fn to_vec(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();

        for path in &self.orderable_set.to_vec() {
            paths.push(path.to_path_buf());
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // It is possible this test can be deleted, since to_vec() is being tested implicitly in the
    // other tests
    #[test]
    fn to_vec_test() {
        let path_to_folder: &Path = Path::new("./test_files");
        let all_files = FileSet::new(path_to_folder);

        let file_vec = all_files.to_vec();
        let directory_location = file_vec[0].parent().unwrap();

        assert_eq!(file_vec.len(), 9);
        assert!(file_vec.contains(&directory_location.join(".DS_Store")));
        assert!(file_vec.contains(&directory_location.join(".hidden_file_1.txt")));
        assert!(file_vec.contains(&directory_location.join(".hidden_file_2")));
        assert!(file_vec.contains(&directory_location.join(".symlink_to_gitkeep")));
        assert!(file_vec.contains(&directory_location.join("cat.doc")));
        assert!(file_vec.contains(&directory_location.join("directory_1")));
        assert!(file_vec.contains(&directory_location.join("directory_2")));
        assert!(file_vec.contains(&directory_location.join("dog.txt")));
        assert!(file_vec.contains(&directory_location.join("video.mov")));
    }

    #[test]
    fn visibility_filter_test() {
        let path_to_folder: &Path = Path::new("./test_files");
        let mut all_files = FileSet::new(path_to_folder);

        let hidden_files = all_files
            .filter(Filter::Visibility(VisibilityFilter::Hidden))
            .to_vec();
        let directory_location = hidden_files[0].parent().unwrap();

        assert_eq!(hidden_files.len(), 4);
        assert!(hidden_files.contains(&directory_location.join(".DS_Store")));
        assert!(hidden_files.contains(&directory_location.join(".hidden_file_1.txt")));
        assert!(hidden_files.contains(&directory_location.join(".hidden_file_2")));
        assert!(hidden_files.contains(&directory_location.join(".symlink_to_gitkeep")));

        let visible_files = all_files
            .filter(Filter::Visibility(VisibilityFilter::Visible))
            .to_vec();

        assert_eq!(visible_files.len(), 5);
        assert!(visible_files.contains(&directory_location.join("cat.doc")));
        assert!(visible_files.contains(&directory_location.join("directory_1")));
        assert!(visible_files.contains(&directory_location.join("directory_2")));
        assert!(visible_files.contains(&directory_location.join("dog.txt")));
        assert!(visible_files.contains(&directory_location.join("video.mov")));
    }

    #[test]
    fn item_filter_test() {
        let path_to_folder: &Path = Path::new("./test_files");
        let mut all_files = FileSet::new(path_to_folder);

        let directories = all_files
            .filter(Filter::Item(ItemFilter::Directory))
            .to_vec();
        let directory_location = directories[0].parent().unwrap();

        assert_eq!(directories.len(), 2);
        assert!(directories.contains(&directory_location.join("directory_1")));
        assert!(directories.contains(&directory_location.join("directory_2")));

        let files = all_files.filter(Filter::Item(ItemFilter::File)).to_vec();

        assert_eq!(files.len(), 6);
        assert!(files.contains(&directory_location.join(".DS_Store")));
        assert!(files.contains(&directory_location.join(".hidden_file_1.txt")));
        assert!(files.contains(&directory_location.join(".hidden_file_2")));
        assert!(files.contains(&directory_location.join("cat.doc")));
        assert!(files.contains(&directory_location.join("dog.txt")));
        assert!(files.contains(&directory_location.join("video.mov")));

        let symlinks = all_files.filter(Filter::Item(ItemFilter::Symlink)).to_vec();

        assert_eq!(symlinks.len(), 1);
        assert!(symlinks.contains(&directory_location.join(".symlink_to_gitkeep")));
    }

    #[test]
    fn exclude_test_1() {
        let path_to_folder: &Path = Path::new("./test_files");
        let mut all_files = FileSet::new(path_to_folder);

        let all_but_symlinks = all_files
            .exclude(Filter::Item(ItemFilter::Symlink))
            .to_vec();
        let directory_location = all_but_symlinks[0].parent().unwrap();

        assert_eq!(all_but_symlinks.len(), 8);
        assert!(all_but_symlinks.contains(&directory_location.join(".DS_Store")));
        assert!(all_but_symlinks.contains(&directory_location.join(".hidden_file_1.txt")));
        assert!(all_but_symlinks.contains(&directory_location.join(".hidden_file_2")));
        assert!(all_but_symlinks.contains(&directory_location.join("cat.doc")));
        assert!(all_but_symlinks.contains(&directory_location.join("directory_1")));
        assert!(all_but_symlinks.contains(&directory_location.join("directory_2")));
        assert!(all_but_symlinks.contains(&directory_location.join("dog.txt")));
        assert!(all_but_symlinks.contains(&directory_location.join("video.mov")));
    }

    #[test]
    fn exclude_test_2() {
        let path_to_folder: &Path = Path::new("./test_files");
        let mut all_files = FileSet::new(path_to_folder);

        let all_visible_files = all_files
            .exclude(Filter::Visibility(VisibilityFilter::Hidden))
            .to_vec();
        let directory_location = all_visible_files[0].parent().unwrap();

        assert_eq!(all_visible_files.len(), 5);
        assert!(all_visible_files.contains(&directory_location.join("cat.doc")));
        assert!(all_visible_files.contains(&directory_location.join("directory_1")));
        assert!(all_visible_files.contains(&directory_location.join("directory_2")));
        assert!(all_visible_files.contains(&directory_location.join("dog.txt")));
        assert!(all_visible_files.contains(&directory_location.join("video.mov")));
    }
}

// Ordering
// Size (greater than / less than)
// Sort test / reverse test / excluding
