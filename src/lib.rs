use std::fs::read_dir;
use std::path::{Path, PathBuf};

use indexmap::IndexSet;

mod enums;
pub use enums::{Comparison, Filter, ItemFilter, OrderBy, SizeFilter, VisibilityFilter};

pub struct FileSet {
    index_set: IndexSet<PathBuf>,
}

impl FileSet {
    pub fn new(directory: &Path) -> FileSet {
        FileSet {
            index_set: read_dir(&directory)
                .unwrap()
                .map(|x| x.unwrap().path())
                .collect::<IndexSet<_>>(),
        }
    }

    pub fn exclude(&mut self, filter: Filter) -> FileSet {
        let items_to_exclude: FileSet = self.filter(filter);

        FileSet {
            index_set: self
                .index_set
                .difference(&items_to_exclude.index_set)
                .cloned()
                .collect::<IndexSet<_>>(),
        }
    }

    pub fn filter(&mut self, filter: Filter) -> FileSet {
        FileSet {
            index_set: match filter {
                Filter::Item(item) => self.filter_by_item(item),
                Filter::Visibility(visibility) => self.filter_by_visibility(visibility),
            },
        }
    }

    fn filter_by_item(&mut self, item_filter: ItemFilter) -> IndexSet<PathBuf> {
        let vec_iter = self.index_set.clone().into_iter();

        // TODO: Make a macro for x.symlink_metadata().unwrap().file_type() ?
        // TODO: Try to remove factor out collect()
        match item_filter {
            ItemFilter::Directory => vec_iter
                .filter(|x| x.symlink_metadata().unwrap().file_type().is_dir())
                .collect::<IndexSet<_>>(),
            ItemFilter::File => vec_iter
                .filter(|x| x.symlink_metadata().unwrap().file_type().is_file())
                .collect::<IndexSet<_>>(),
            ItemFilter::Symlink => vec_iter
                .filter(|x| x.symlink_metadata().unwrap().file_type().is_symlink())
                .collect::<IndexSet<_>>(),
        }
    }

    fn filter_by_visibility(
        &mut self,
        visibility_filter: VisibilityFilter,
    ) -> IndexSet<PathBuf> {
        let should_find_visible_files: bool = match visibility_filter {
            VisibilityFilter::Hidden => false,
            VisibilityFilter::Visible => true,
        };

        self.index_set
            .clone()
            .into_iter()
            .filter(|x| {
                should_find_visible_files
                    != x.file_name().unwrap().to_string_lossy().starts_with('.')
            })
            .collect::<IndexSet<_>>()
    }

    pub fn reverse(&mut self) -> FileSet {
        FileSet {
            index_set: self
                .index_set
                .clone()
                .into_iter()
                .rev()
                .collect::<IndexSet<_>>(),
        }
    }

    pub fn to_vec(&self) -> Vec<PathBuf> {
        self.index_set.clone().into_iter().map(|x| x).collect()
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
