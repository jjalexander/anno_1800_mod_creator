use std::path::PathBuf;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub(crate) struct Identifier {
    pub(crate) file_path: PathBuf,
    pub(crate) kind: Kind,
    pub(crate) value: String,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub(crate) enum Kind {
    XPath,
    Name,
    GUID,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub(crate) enum ParentIdentifier {
    None,
    DefaultValues,
    Template(String),
    GUID(String),
}
