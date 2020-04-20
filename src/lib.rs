use std::fs::{read_dir, FileType};
use std::path::{Path, PathBuf};

use indexmap::IndexSet;

mod enums;
pub use enums::{Comparison, Filter, ItemFilter, OrderBy, SizeFilter, VisibilityFilter};

pub struct FileSet {
    index_set: IndexSet<PathBuf>,
}

impl FileSet {
    pub fn new(directory: PathBuf) -> FileSet {
        FileSet {
            index_set: read_dir(&directory)
                .unwrap()
                .map(|x| x.unwrap().path())
                .collect::<IndexSet<PathBuf>>(),
        }
    }

    pub fn exclude(&self, filter: Filter) -> FileSet {
        let items_to_exclude: FileSet = self.filter(filter);

        FileSet {
            index_set: self
                .index_set
                .difference(&items_to_exclude.index_set)
                .cloned()
                .collect::<IndexSet<PathBuf>>(),
        }
    }

    pub fn filter(&self, filter: Filter) -> FileSet {
        FileSet {
            index_set: match filter {
                Filter::Item(item) => self.filter_by_item(item),
                Filter::Visibility(visibility) => self.filter_by_visibility(visibility),
            },
        }
    }

    fn filter_by_item(&self, item_filter: ItemFilter) -> IndexSet<PathBuf> {
        let file_type_function = match item_filter {
            ItemFilter::Directory => FileType::is_dir,
            ItemFilter::File => FileType::is_file,
            ItemFilter::Symlink => FileType::is_symlink,
        };

        self.index_set
            .clone()
            .into_iter()
            .filter(|x: &PathBuf| file_type_function(&x.symlink_metadata().unwrap().file_type()))
            .collect::<IndexSet<PathBuf>>()
    }

    fn filter_by_visibility(&self, visibility_filter: VisibilityFilter) -> IndexSet<PathBuf> {
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
            .collect::<IndexSet<PathBuf>>()
    }

    // Try to refactor match arms like filter_by_item()
    // Make each match arm a function, if necessary
    // Is there a better way to handle combining the items other than union()?
    pub fn order_by(&self, order_by: OrderBy) -> FileSet {
        let mut index_set: IndexSet<PathBuf>;

        match order_by {
            OrderBy::Extension | OrderBy::Name | OrderBy::Size => {
                index_set = self.index_set.clone();

                index_set.sort_by(|a, b| match order_by {
                    OrderBy::Extension => Ord::cmp(&a.extension(), &b.extension()),
                    OrderBy::Name => Ord::cmp(&a.file_name(), &b.file_name()),
                    _ => {
                        let get_file_size =
                            |a: &Path| -> u64 { a.symlink_metadata().unwrap().len() };
                        Ord::cmp(&get_file_size(&a), &get_file_size(&b))
                    }
                });
            }
            OrderBy::Item => {
                let get_index_set_union = |a: &IndexSet<_>, b: &IndexSet<_>| -> IndexSet<PathBuf> {
                    a.union(&b).cloned().collect::<IndexSet<PathBuf>>()
                };
                let directories = self.filter(Filter::Item(ItemFilter::Directory));
                let files = self.filter(Filter::Item(ItemFilter::File));
                let symlinks = self.filter(Filter::Item(ItemFilter::Symlink));

                index_set = get_index_set_union(&directories.index_set, &files.index_set);
                index_set = get_index_set_union(&index_set, &symlinks.index_set);
            }
        }

        FileSet { index_set }
    }

    pub fn reverse(&self) -> FileSet {
        FileSet {
            index_set: self
                .index_set
                .clone()
                .into_iter()
                .rev()
                .collect::<IndexSet<PathBuf>>(),
        }
    }

    pub fn to_vec(&self) -> Vec<PathBuf> {
        self.index_set.clone().into_iter().map(|x| x).collect()
    }

    pub fn len(&self) -> usize {
        self.index_set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index_set.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // It is possible this test can be deleted, since to_vec() is being tested implicitly in the
    // other tests
    #[test]
    fn to_vec_test() {
        let file_vec = FileSet::new(PathBuf::from("./test_files")).to_vec();
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
        let all_files = FileSet::new(PathBuf::from("./test_files"));

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
        let all_files = FileSet::new(PathBuf::from("./test_files"));
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
        let all_items_but_symlinks = FileSet::new(PathBuf::from("./test_files"))
            .exclude(Filter::Item(ItemFilter::Symlink))
            .to_vec();
        let directory_location = all_items_but_symlinks[0].parent().unwrap();

        assert_eq!(all_items_but_symlinks.len(), 8);
        assert!(all_items_but_symlinks.contains(&directory_location.join(".DS_Store")));
        assert!(all_items_but_symlinks.contains(&directory_location.join(".hidden_file_1.txt")));
        assert!(all_items_but_symlinks.contains(&directory_location.join(".hidden_file_2")));
        assert!(all_items_but_symlinks.contains(&directory_location.join("cat.doc")));
        assert!(all_items_but_symlinks.contains(&directory_location.join("directory_1")));
        assert!(all_items_but_symlinks.contains(&directory_location.join("directory_2")));
        assert!(all_items_but_symlinks.contains(&directory_location.join("dog.txt")));
        assert!(all_items_but_symlinks.contains(&directory_location.join("video.mov")));
    }

    #[test]
    fn exclude_test_2() {
        let all_visible_files = FileSet::new(PathBuf::from("./test_files"))
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

    #[test]
    fn order_by_extension_test() {
        let items_ordered_by_extension = FileSet::new(PathBuf::from("./test_files"))
            .order_by(OrderBy::Extension)
            .to_vec();

        assert_eq!(items_ordered_by_extension.len(), 9);
        assert!(items_ordered_by_extension[0].extension().is_none());
        assert!(items_ordered_by_extension[1].extension().is_none());
        assert!(items_ordered_by_extension[2].extension().is_none());
        assert!(items_ordered_by_extension[3].extension().is_none());
        assert!(items_ordered_by_extension[4].extension().is_none());
        assert!(items_ordered_by_extension[5]
            .extension()
            .unwrap()
            .to_string_lossy()
            .ends_with("doc"));
        assert!(items_ordered_by_extension[6]
            .extension()
            .unwrap()
            .to_string_lossy()
            .ends_with("mov"));
        assert!(items_ordered_by_extension[7]
            .extension()
            .unwrap()
            .to_string_lossy()
            .ends_with("txt"));
        assert!(items_ordered_by_extension[8]
            .extension()
            .unwrap()
            .to_string_lossy()
            .ends_with("txt"));
    }

    #[test]
    fn order_by_item_test() {
        let items_ordered_by_extension = FileSet::new(PathBuf::from("./test_files"))
            .order_by(OrderBy::Item)
            .to_vec();

        assert_eq!(items_ordered_by_extension.len(), 9);
        assert!(items_ordered_by_extension[0].is_dir());
        assert!(items_ordered_by_extension[1].is_dir());
        assert!(items_ordered_by_extension[2].is_file());
        assert!(items_ordered_by_extension[3].is_file());
        assert!(items_ordered_by_extension[4].is_file());
        assert!(items_ordered_by_extension[5].is_file());
        assert!(items_ordered_by_extension[6].is_file());
        assert!(items_ordered_by_extension[7].is_file());
        assert!(items_ordered_by_extension[8]
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn order_by_name_test() {
        let items_ordered_by_extension = FileSet::new(PathBuf::from("./test_files"))
            .order_by(OrderBy::Name)
            .to_vec();

        assert_eq!(items_ordered_by_extension.len(), 9);
        assert_eq!(
            items_ordered_by_extension[0].file_name().unwrap(),
            ".DS_Store"
        );
        assert_eq!(
            items_ordered_by_extension[1].file_name().unwrap(),
            ".hidden_file_1.txt"
        );
        assert_eq!(
            items_ordered_by_extension[2].file_name().unwrap(),
            ".hidden_file_2"
        );
        assert_eq!(
            items_ordered_by_extension[3].file_name().unwrap(),
            ".symlink_to_gitkeep"
        );
        assert_eq!(
            items_ordered_by_extension[4].file_name().unwrap(),
            "cat.doc"
        );
        assert_eq!(
            items_ordered_by_extension[5].file_name().unwrap(),
            "directory_1"
        );
        assert_eq!(
            items_ordered_by_extension[6].file_name().unwrap(),
            "directory_2"
        );
        assert_eq!(
            items_ordered_by_extension[7].file_name().unwrap(),
            "dog.txt"
        );
        assert_eq!(
            items_ordered_by_extension[8].file_name().unwrap(),
            "video.mov"
        );
    }

    #[test]
    fn reverse_test() {
        let items_ordered_by_extension = FileSet::new(PathBuf::from("./test_files"))
            .order_by(OrderBy::Name)
            .reverse()
            .to_vec();

        assert_eq!(items_ordered_by_extension.len(), 9);
        assert_eq!(
            items_ordered_by_extension[0].file_name().unwrap(),
            "video.mov"
        );
        assert_eq!(
            items_ordered_by_extension[1].file_name().unwrap(),
            "dog.txt"
        );
        assert_eq!(
            items_ordered_by_extension[2].file_name().unwrap(),
            "directory_2"
        );
        assert_eq!(
            items_ordered_by_extension[3].file_name().unwrap(),
            "directory_1"
        );
        assert_eq!(
            items_ordered_by_extension[4].file_name().unwrap(),
            "cat.doc"
        );
        assert_eq!(
            items_ordered_by_extension[5].file_name().unwrap(),
            ".symlink_to_gitkeep"
        );
        assert_eq!(
            items_ordered_by_extension[6].file_name().unwrap(),
            ".hidden_file_2"
        );
        assert_eq!(
            items_ordered_by_extension[7].file_name().unwrap(),
            ".hidden_file_1.txt"
        );
        assert_eq!(
            items_ordered_by_extension[8].file_name().unwrap(),
            ".DS_Store"
        );
    }

    #[test]
    fn len_test() {
        let items = FileSet::new(PathBuf::from("./test_files"));
        assert_eq!(items.len(), 9);
    }

    #[test]
    fn is_empty_test() {
        let no_items = FileSet::new(PathBuf::from("./test_files"))
            .filter(Filter::Visibility(VisibilityFilter::Hidden))
            .filter(Filter::Item(ItemFilter::Directory));

        assert!(no_items.is_empty());
    }
}
