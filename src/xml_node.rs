#[derive(Debug)]
pub(crate) enum XmlNode {
    Branch {
        name: String,
        children: Vec<XmlNode>,
    },
    Leaf {
        name: String,
        value: String,
    },
}
