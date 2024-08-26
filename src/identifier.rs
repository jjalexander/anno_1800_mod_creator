#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub(crate) enum Identifier {
    XPath(String),
    Name(String),
    GUID(String),
}
