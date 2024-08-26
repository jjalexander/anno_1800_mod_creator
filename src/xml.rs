pub(crate) enum XmlTag {
    Branch { name: String, children: Vec<XmlTag> },
    Leaf { name: String },
}