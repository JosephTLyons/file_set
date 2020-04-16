use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;

mod orderable_set;
use orderable_set::OrderableSet;

pub enum Filter {
    Item(Item),
    Visibility(Visibility),
}

pub enum Item {
    Directory,
    File,
    Symlink,
}

pub enum OrderBy {
    DirectoryEntryType,
    Extension,
    Name,
    Size,
}

pub enum Size {
    Bytes,
    Kilobytes,
    Megatbytes,
    Gigabytes,
    Terabytes,
}

pub enum Visibility {
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
                Filter::Item(item) => self.filter_by_directory_entry_type(item),
                Filter::Visibility(visibility) => self.filter_by_visibility_type(visibility),
            },
        }
    }

    fn filter_by_directory_entry_type(
        &mut self,
        directory_entry_type: Item,
    ) -> OrderableSet<PathBuf> {
        let mut orderable_set: OrderableSet<PathBuf> = OrderableSet::new();

        match directory_entry_type {
            Item::Directory => {
                for path_buf in self.orderable_set.to_vec() {
                    if path_buf.metadata().unwrap().is_dir() {
                        orderable_set.push(path_buf).unwrap()
                    }
                }
            }
            Item::File => {
                for path_buf in self.orderable_set.to_vec() {
                    if path_buf.metadata().unwrap().is_file() {
                        orderable_set.push(path_buf).unwrap()
                    }
                }
            }
            Item::Symlink => {
                for path_buf in self.orderable_set.to_vec() {
                    if !path_buf.metadata().unwrap().is_dir()
                        && !path_buf.metadata().unwrap().is_file()
                    {
                        orderable_set.push(path_buf).unwrap()
                    }
                }
            }
        }

        orderable_set
    }

    pub fn filter_by_visibility_type(
        &mut self,
        visibility_type: Visibility,
    ) -> OrderableSet<PathBuf> {
        let mut orderable_set: OrderableSet<PathBuf> = OrderableSet::new();

        match visibility_type {
            Visibility::Hidden => {
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
            Visibility::Visible => {
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

    // pub fn exclude(&mut self, filer_type: FilterType) -> FileSet {
    //     // let x = self.filter(filer_type).orderable_set;
    //     // self.orderable_set.difference(x.clone());
    //
    //     self
    // }

    pub fn reverse(&mut self) -> FileSet {
        let mut orderable_set = self.orderable_set.clone();
        orderable_set.reverse();
        FileSet { orderable_set }
    }

    pub fn to_vec(&self) -> Option<Vec<PathBuf>> {
        if self.orderable_set.to_vec().is_empty() {
            return None;
        }

        let mut paths: Vec<PathBuf> = Vec::new();

        for path in &self.orderable_set.to_vec() {
            paths.push(path.to_path_buf());
        }

        Some(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_vec_test() {
        let path_to_folder: &Path = Path::new("./src/test_files");
        let all_files = FileSet::new(path_to_folder);

        let file_vec = all_files.to_vec().unwrap();
        let directory_location = file_vec[0].parent().unwrap();

        assert_eq!(file_vec.len(), 8);
        assert!(file_vec.contains(&directory_location.join(".hidden_file_1.txt")));
        assert!(file_vec.contains(&directory_location.join(".hidden_file_2")));
        assert!(file_vec.contains(&directory_location.join(".DS_Store")));
        assert!(file_vec.contains(&directory_location.join("cat.doc")));
        assert!(file_vec.contains(&directory_location.join("directory_1")));
        assert!(file_vec.contains(&directory_location.join("directory_2")));
        assert!(file_vec.contains(&directory_location.join("dog.txt")));
        assert!(file_vec.contains(&directory_location.join("video.mov")));
    }

    #[test]
    fn visibility_type_filter_test() {
        let path_to_folder: &Path = Path::new("./src/test_files");
        let mut all_files = FileSet::new(path_to_folder);

        let hidden_files = all_files
            .filter(Filter::Visibility(Visibility::Hidden))
            .to_vec()
            .unwrap();
        let directory_location = hidden_files[0].parent().unwrap();

        assert_eq!(hidden_files.len(), 3);
        assert!(hidden_files.contains(&directory_location.join(".DS_Store")));
        assert!(hidden_files.contains(&directory_location.join(".hidden_file_1.txt")));
        assert!(hidden_files.contains(&directory_location.join(".hidden_file_2")));

        let visible_files = all_files
            .filter(Filter::Visibility(Visibility::Visible))
            .to_vec()
            .unwrap();

        assert_eq!(visible_files.len(), 5);
        assert!(visible_files.contains(&directory_location.join("cat.doc")));
        assert!(visible_files.contains(&directory_location.join("directory_1")));
        assert!(visible_files.contains(&directory_location.join("directory_2")));
        assert!(visible_files.contains(&directory_location.join("dog.txt")));
        assert!(visible_files.contains(&directory_location.join("video.mov")));
    }

    #[test]
    fn directory_entry_type_filter_test() {
        let path_to_folder: &Path = Path::new("./src/test_files");
        let mut all_files = FileSet::new(path_to_folder);

        let directories = all_files
            .filter(Filter::Item(Item::Directory))
            .to_vec()
            .unwrap();
        let directory_location = directories[0].parent().unwrap();

        assert_eq!(directories.len(), 2);
        assert!(directories.contains(&directory_location.join("directory_1")));
        assert!(directories.contains(&directory_location.join("directory_2")));

        let files = all_files.filter(Filter::Item(Item::File)).to_vec().unwrap();

        assert_eq!(files.len(), 6);
        assert!(files.contains(&directory_location.join(".hidden_file_1.txt")));
        assert!(files.contains(&directory_location.join(".hidden_file_2")));
        assert!(files.contains(&directory_location.join(".DS_Store")));
        assert!(files.contains(&directory_location.join("cat.doc")));
        assert!(files.contains(&directory_location.join("dog.txt")));
        assert!(files.contains(&directory_location.join("video.mov")));
    }
}

// Ordering
// Size (greater than / less than)
// Reverse
// Sort by file name, extension name
// Files only
// Directorys only
// Extension name
// Exclude
