pub enum Filter {
    Item(ItemFilter),
    // Size(SizeFilter),
    Text(TextFilterBy, &'static str),
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

pub enum TextFilterBy {
    Extension,
    Name,
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
