use roxmltree::Node;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::{xml_node::XmlNode, xml_structure::XmlTag};

pub(crate) fn get_paths(path: &PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut properties_paths = Vec::new();
    let mut templates_paths = Vec::new();
    let mut assets_paths = Vec::new();

    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .for_each(|entry| match entry.file_name().to_str() {
            Some("properties.xml") => properties_paths.push(entry.path().to_path_buf()),
            Some("templates.xml") => templates_paths.push(entry.path().to_path_buf()),
            Some("assets.xml") => assets_paths.push(entry.path().to_path_buf()),
            _ => (),
        });

    properties_paths.sort();
    templates_paths.sort();
    assets_paths.sort();

    (properties_paths, templates_paths, assets_paths)
}

pub(crate) fn get_xpath(node: &roxmltree::Node) -> String {
    node.ancestors()
        .filter(|node| !node.is_root())
        .map(|node| {
            let tag_name = node.tag_name().name().to_string();
            let index = node
                .parent()
                .unwrap()
                .children()
                .filter(|child| child.tag_name().name() == tag_name)
                .position(|child| child == node)
                .unwrap();
            format!("{}[{}]", tag_name, index + 1)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .fold(String::new(), |acc, x| format!("{}/{}", acc, x))
}

pub(crate) fn extract_xml(node: &Node, xml_structure: &XmlTag) -> Option<XmlNode> {
    let tag_name = xml_structure.get_name();

    let children_nodes_with_tag_name = node
        .children()
        .filter(|child| child.tag_name().name() == tag_name)
        .collect::<Vec<_>>();

    if children_nodes_with_tag_name.is_empty() {
        return None;
    }
    if children_nodes_with_tag_name.len() > 1 {
        panic!("More than one child with the tag name {} in xpath {}", tag_name, get_xpath(node));
    }

    let child_node = children_nodes_with_tag_name.first().unwrap();

    match xml_structure {
        XmlTag::Branch { children, .. } => {
            let children = children
                .iter()
                .filter_map(|child| extract_xml(child_node, child))
                .collect();
            Some(XmlNode::Branch {
                name: tag_name.to_string(),
                children,
            })
        }
        XmlTag::Leaf { .. } => Some(XmlNode::Leaf {
            name: tag_name.to_string(),
            value: child_node.text().unwrap().to_string(),
        }),
    }
}
