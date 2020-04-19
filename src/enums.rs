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

pub enum Comparison {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

pub enum OrderBy {
    Extension,
    Item,
    Name,
    Size,
}
