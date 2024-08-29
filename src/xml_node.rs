#[derive(Debug, Clone)]
pub(crate) struct XmlNode {
    pub(crate) name: String,
    pub(crate) present: bool,
    pub(crate) data: XmlNodeData,
}

#[derive(Debug, Clone)]
pub(crate) enum XmlNodeData {
    Branch(Vec<XmlNode>),
    Leaf(String),
    None,
}
