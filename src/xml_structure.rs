#[derive(Clone)]
pub(crate) struct XmlTag {
    pub(crate) name: String,
    pub(crate) content: Content,
}

#[derive(Clone)]
pub(crate) enum Content {
    Branch(Vec<XmlTag>),
    Leaf,
}
