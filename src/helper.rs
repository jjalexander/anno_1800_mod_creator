use crate::{
    identifier::{Identifier, ParentIdentifier},
    state::State,
    xml_node::{XmlNode, XmlNodeData},
    xml_structure::{Content, XmlTag},
};
use itertools::Itertools;
use roxmltree::Node;
use std::io::Write;
use std::{collections::HashMap, path::PathBuf};
use walkdir::WalkDir;

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

pub(crate) fn has_properties_child(node: &Node<'_, '_>, query: &XmlTag) -> bool {
    let Some(properties_node) = node
        .children()
        .filter(|child| child.tag_name().name() == "Properties")
        .at_most_one()
        .expect(
            format!(
                "More than one Properties node found in node {}",
                get_xpath(&node)
            )
            .as_str(),
        )
    else {
        return false;
    };

    has_direct_child(&properties_node, query)
}

pub(crate) fn has_direct_child(node: &Node<'_, '_>, query: &XmlTag) -> bool {
    node.children()
        .filter(|child| child.tag_name().name() == query.name)
        .count()
        == 1
}

pub(crate) fn extract_content_from_values(
    node: &Node<'_, '_>,
    query: &XmlTag,
    parent_content: Option<&XmlNode>,
) -> Option<XmlNode> {
    let values_node = node
        .children()
        .filter(|child| child.tag_name().name() == "Values")
        .at_most_one()
        .expect(
            format!(
                "More than one Values node found in node {}",
                get_xpath(&node)
            )
            .as_str(),
        )
        .expect(format!("No Values node found in node {}", get_xpath(&node)).as_str());

    extract_content(&values_node, query, parent_content)
}

pub(crate) fn extract_content_from_properties(
    node: &Node<'_, '_>,
    query: &XmlTag,
    parent_content: Option<&XmlNode>,
) -> Option<XmlNode> {
    let Some(properties_node) = node
        .children()
        .filter(|child| child.tag_name().name() == "Properties")
        .at_most_one()
        .expect(
            format!(
                "More than one Properties node found in node {}",
                get_xpath(&node)
            )
            .as_str(),
        )
    else {
        return create_content(query, parent_content);
    };

    extract_content(&properties_node, query, parent_content)
}

pub(crate) fn extract_content(
    node: &Node<'_, '_>,
    query: &XmlTag,
    parent_content: Option<&XmlNode>,
) -> Option<XmlNode> {
    let Some(child_node) = node
        .children()
        .filter(|child| child.tag_name().name() == query.name)
        .at_most_one()
        .expect(
            format!(
                "More than one {} node found in node {}",
                query.name,
                get_xpath(&node)
            )
            .as_str(),
        )
    else {
        return create_content(query, parent_content);
    };

    match &query.content {
        Content::Branch(query_children) => {
            let created_children = query_children
                .iter()
                .filter_map(|query_child| {
                    let parent_content_child = match parent_content {
                        Some(parent_content) => match &parent_content.data {
                            XmlNodeData::Branch(children) => {
                                children.iter().find(|child| child.name == query_child.name)
                            }
                            _ => None,
                        },
                        None => None,
                    };
                    extract_content(&child_node, query_child, parent_content_child)
                })
                .collect::<Vec<_>>();
            Some(XmlNode {
                name: query.name.clone(),
                present: true,
                data: XmlNodeData::Branch(created_children),
            })
        }
        Content::Leaf => {
            let Some(text) = child_node.text().and_then(|text| Some(text.to_string())) else {
                return None;
            };
            Some(XmlNode {
                name: query.name.clone(),
                present: true,
                data: XmlNodeData::Leaf(text),
            })
        }
    }
}

fn create_content(query: &XmlTag, parent_content: Option<&XmlNode>) -> Option<XmlNode> {
    match &query.content {
        Content::Branch(query_children) => {
            let created_children = query_children
                .iter()
                .filter_map(|query_child| {
                    let parent_content_child = match parent_content {
                        Some(parent_content) => match &parent_content.data {
                            XmlNodeData::Branch(children) => {
                                children.iter().find(|child| child.name == query_child.name)
                            }
                            _ => None,
                        },
                        None => None,
                    };
                    create_content(query_child, parent_content_child)
                })
                .collect::<Vec<_>>();
            return Some(XmlNode {
                name: query.name.clone(),
                present: false,
                data: XmlNodeData::Branch(created_children.to_vec()),
            });
        }
        Content::Leaf => {
            return Some(XmlNode {
                name: query.name.clone(),
                present: false,
                data: parent_content
                    .map(|parent_content| match &parent_content.data {
                        XmlNodeData::Branch(_) => XmlNodeData::None,
                        XmlNodeData::Leaf(x) => XmlNodeData::Leaf(x.clone()),
                        XmlNodeData::None => XmlNodeData::None,
                    })
                    .unwrap_or(XmlNodeData::None),
            });
        }
    }
}

pub(crate) fn create_mod(
    output_path: &PathBuf,
    mod_name: &str,
    identifiers: &Vec<Identifier>,
    identifiers_as_parent: &HashMap<ParentIdentifier, Identifier>,
    parent_identifiers: &HashMap<Identifier, ParentIdentifier>,
    states: &HashMap<Identifier, State>,
    contents: &HashMap<Identifier, XmlNode>,
) {
    let mod_path = output_path.join(enhanced_name(mod_name));

    // delete_mod_files(&mod_path);

    // create_mod_directory(&mod_path);

    // let mut path_vs_mod_ops: HashMap<String, Vec<ModOpsStructure>> = HashMap::new();

    identifiers.into_iter().for_each(|identifier| {
        let file_path = identifier.file_path.clone();
        let content = contents.get(identifier).unwrap();
        let state = states.get(identifier).unwrap();

        let mod_ops = create_mod_ops(content, state);

        match mod_ops {
            ModOpsStructure::None => return,
            _ => println!("{:?} - {:?}", identifier, mod_ops),
        }

        // path_vs_mod_ops
        //     .entry(file_path)
        //     .or_insert_with(Vec::new)
        //     .push(mod_ops);
    });

    // path_vs_mod_ops.iter().for_each(|(path, mod_ops)| {
    //     let full_path = mod_path.join(path);
    //     let parent_path = full_path.parent().unwrap();
    //     std::fs::create_dir_all(parent_path).unwrap();
    //     std::fs::File::create(&full_path).unwrap();
    //     let mut file = std::fs::OpenOptions::new()
    //         .write(true)
    //         .open(full_path)
    //         .unwrap();
    //     mod_ops.iter().for_each(|mod_op| {
    //         writeln!(file, "{:?}", mod_op).unwrap();
    //     });
    // });
}

fn create_mod_ops(content: &XmlNode, state: &State) -> ModOpsStructure {
    let y = match &content.data {
        XmlNodeData::Branch(children) => {
            let child_mod_ops = children
                .iter()
                .map(|child| create_mod_ops(child, state))
                .filter(|mod_ops| match mod_ops {
                    ModOpsStructure::None => false,
                    _ => true,
                })
                .collect::<Vec<_>>();
            match child_mod_ops.is_empty() {
                true => ModOpsStructure::None,
                false => match content.present {
                    true => ModOpsStructure::Leafs(child_mod_ops),
                    false => ModOpsStructure::Branch(
                        format!("Add intermediate node {}", content.name),
                        child_mod_ops,
                    ),
                },
            }
        }
        XmlNodeData::Leaf(old_value) => match (state, content.present) {
            (State::Included, true) => ModOpsStructure::Leaf(format!("Replace {}", content.name)),
            (State::Included, false) => {
                ModOpsStructure::Leaf(format!("Allow {} from ancestor", content.name))
            }
            (State::Excluded, true) => ModOpsStructure::Leaf(format!("Keep {}", content.name)),
            (State::Excluded, false) => {
                ModOpsStructure::Leaf(format!("Enforce {} from ancestor", content.name))
            }
            (State::ExcludedByAncestor, true) => {
                ModOpsStructure::Leaf(format!("Keep {}", content.name))
            }
            (State::ExcludedByAncestor, false) => {
                ModOpsStructure::Leaf(format!("Keep {} from ancestor", content.name))
            }
            (State::Forced, true) => {
                ModOpsStructure::Leaf(format!("Change {} to new value", content.name))
            }
            (State::Forced, false) => {
                ModOpsStructure::Leaf(format!("Set {} to new value", content.name))
            }
            (State::ForcedByAncestor, true) => {
                ModOpsStructure::Leaf(format!("Change {}", content.name))
            }
            (State::ForcedByAncestor, false) => {
                ModOpsStructure::Leaf(format!("Allow {} from ancestor", content.name))
            }
        },
        XmlNodeData::None => ModOpsStructure::None,
    };

    y
}

#[derive(PartialEq, Debug)]
enum ModOpsStructure {
    Branch(String, Vec<ModOpsStructure>),
    Leafs(Vec<ModOpsStructure>),
    Leaf(String),
    None,
}

fn create_mod_directory(mod_path: &PathBuf) {
    std::fs::create_dir(&mod_path).unwrap();
}

pub(crate) fn delete_mod_files(mod_path: &PathBuf) {
    let Ok(_) = std::fs::remove_dir_all(mod_path) else {
        return;
    };
}

fn enhanced_name(mod_name: &str) -> String {
    format!("JJ's Enhanced {}", mod_name)
}
