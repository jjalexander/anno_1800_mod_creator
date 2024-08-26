pub(crate) enum XmlTag {
    Branch { name: String, children: Vec<XmlTag> },
    Leaf { name: String },
}

impl XmlTag {
    pub(crate) fn get_name(&self) -> &str {
        match self {
            XmlTag::Branch { name, .. } => name,
            XmlTag::Leaf { name } => name,
        }
    }
}