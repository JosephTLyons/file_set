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

pub enum OrderByFilter {
    // Just sort all of them ascending, then the reverse can be applied
    Extension,
    Name,
    Size,
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

    pub fn filter(&mut self, filer: Filter) -> FileSet {
        FileSet {
            orderable_set: match filer {
                Filter::Item(item) => self.filter_by_item(item),
                Filter::Visibility(visibility) => self.filter_by_visibility(visibility),
            },
        }
    }

    fn filter_by_item(&mut self, item_filter: ItemFilter) -> OrderableSet<PathBuf> {
        let mut orderable_set: OrderableSet<PathBuf> = OrderableSet::new();

        match item_filter {
            ItemFilter::Directory => {
                for path_buf in self.orderable_set.to_vec() {
                    if path_buf.metadata().unwrap().file_type().is_dir() {
                        orderable_set.push(path_buf).unwrap()
                    }
                }
            }
            ItemFilter::File => {
                for path_buf in self.orderable_set.to_vec() {
                    if path_buf.symlink_metadata().unwrap().file_type().is_file() {
                        orderable_set.push(path_buf).unwrap()
                    }
                }
            }
            ItemFilter::Symlink => {
                for path_buf in self.orderable_set.to_vec() {
                    if path_buf
                        .symlink_metadata()
                        .unwrap()
                        .file_type()
                        .is_symlink()
                    {
                        orderable_set.push(path_buf).unwrap()
                    }
                }
            }
        }

        orderable_set
    }

    pub fn filter_by_visibility(
        &mut self,
        visibility_filter: VisibilityFilter,
    ) -> OrderableSet<PathBuf> {
        let mut orderable_set: OrderableSet<PathBuf> = OrderableSet::new();

        match visibility_filter {
            VisibilityFilter::Hidden => {
                // let x: Vec<PathBuf> = self
                //     .orderable_set
                //     .to_vec()
                //     .into_iter()
                //     .filter(|&x| x.to_string_lossy().starts_with('.'))
                //     .collect();
                // let y = OrderableSet::try_from(x);

                for path_buf in self.orderable_set.to_vec() {
                    if path_buf
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with('.')
                    {
                        orderable_set.push(path_buf).unwrap();
                    }
                }
            }
            VisibilityFilter::Visible => {
                for path_buf in self.orderable_set.to_vec() {
                    if !path_buf
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with('.')
                    {
                        orderable_set.push(path_buf).unwrap();
                    }
                }
            }
        }

        orderable_set
    }

    // pub fn exclude(&mut self, filter: Filter) -> FileSet {
    // }

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
}

// Ordering
// Size (greater than / less than)
// Sort test / reverse test / excluding
