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
    // Just sort all of them ascending, then the reverse can be applied
    Extension,
    Item,
    Name,
    Size,
}
