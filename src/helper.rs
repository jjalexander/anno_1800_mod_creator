use crate::{
    identifier::{self, Identifier},
    state::State,
    xml_node::{XmlNode, XmlNodeData},
    xml_structure::{Content, XmlTag},
    NodeType,
};
use core::panic;
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

pub(crate) fn write_mod(
    output_path: &PathBuf,
    mod_name: &str,
    identifiers: &Vec<Identifier>,
    node_types: &HashMap<Identifier, NodeType>,
    states: &HashMap<Identifier, State>,
    contents: &HashMap<Identifier, XmlNode>,
) {
    let mod_path = output_path.join(enhanced_name(mod_name));

    delete_mod_files(&mod_path);

    create_mod_directory(&mod_path);

    let mut path_vs_mod_ops: HashMap<PathBuf, Vec<ModOp>> = HashMap::new();

    identifiers.into_iter().for_each(|identifier| {
        let file_path = identifier.file_path.clone();
        let content = contents.get(identifier).unwrap();
        let state = states.get(identifier).unwrap();
        let node_type = node_types.get(identifier).unwrap();

        let mod_ops_structure = create_mod_ops_structure(content, state);

        if !are_any_changes_required(&mod_ops_structure) {
            return;
        }

        let mod_op_path_root = match identifier.kind {
            identifier::Kind::XPath => match node_type {
                NodeType::DefaultValues => format!("{}", identifier.value),
                NodeType::Asset => format!("{}/Values", identifier.value),
                _ => {
                    panic!(
                        "Unsupported node type for XPath identifier: {:?}",
                        node_type
                    );
                }
            },

            identifier::Kind::Name => format!("//Template[Name='{}']/Properties", identifier.value),
            identifier::Kind::GUID => format!(
                "//Asset[Values/Standard/GUID = '{}']/Values",
                identifier.value
            ),
        };

        let mod_ops = convert_mod_ops_structure_to_mod_ops(mod_op_path_root, &mod_ops_structure);

        path_vs_mod_ops
            .entry(file_path)
            .or_insert_with(Vec::new)
            .extend(mod_ops);
    });

    path_vs_mod_ops.iter().for_each(|(path, mod_ops)| {
        let full_path = mod_path.join(path);
        let parent_path = full_path.parent().unwrap();
        std::fs::create_dir_all(parent_path).unwrap();
        std::fs::File::create(&full_path).unwrap();
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(full_path)
            .unwrap();
        writeln!(file, "<ModOps>").unwrap();
        mod_ops.iter().for_each(|mod_op| {
            mod_op.to_xml().iter().for_each(|line| {
                writeln!(file, "  {}", line).unwrap();
            });
        });
        writeln!(file, "</ModOps>").unwrap();
    });
}

fn convert_mod_ops_structure_to_mod_ops(
    mod_op_root_path: String,
    mod_ops_structure: &ModOpsStructure,
) -> Vec<ModOp> {
    let mut mod_ops = Vec::new();

    match &mod_ops_structure.kind {
        ModOpsKind::ReplaceValue(value) => mod_ops.push(ModOp {
            mod_op_type: "Replace".to_string(),
            mod_op_path: format!("{}/{}", mod_op_root_path, mod_ops_structure.name),
            mod_op_value: format!("<{0}>{1}</{0}>", mod_ops_structure.name, value),
        }),
        ModOpsKind::AddValue(value) => mod_ops.push(ModOp {
            mod_op_type: "Add".to_string(),
            mod_op_path: mod_op_root_path.clone(),
            mod_op_value: format!("<{0}>{1}</{0}>", mod_ops_structure.name, value),
        }),
        ModOpsKind::AddNode => mod_ops.push(ModOp {
            mod_op_type: "Add".to_string(),
            mod_op_path: mod_op_root_path.clone(),
            mod_op_value: format!("<{0}></{0}>", mod_ops_structure.name),
        }),
        ModOpsKind::None => (),
    }

    mod_ops_structure.children.iter().for_each(|child| {
        mod_ops.extend(convert_mod_ops_structure_to_mod_ops(
            format!("{}/{}", mod_op_root_path, mod_ops_structure.name),
            child,
        ))
    });

    mod_ops
}

#[derive(Debug)]
struct ModOp {
    mod_op_type: String,
    mod_op_path: String,
    mod_op_value: String,
}

impl ModOp {
    fn to_xml(&self) -> Vec<String> {
        let mut xml = Vec::new();
        xml.push(format!(
            "<ModOp Type=\"{0}\" Path=\"{1}\">",
            self.mod_op_type, self.mod_op_path
        ));
        xml.push(format!("  {}", self.mod_op_value));
        xml.push("</ModOp>".to_string());
        xml
    }
}

#[derive(PartialEq, Debug)]
struct ModOpsStructure {
    name: String,
    kind: ModOpsKind,
    children: Vec<ModOpsStructure>,
}

#[derive(PartialEq, Debug)]
enum ModOpsKind {
    ReplaceValue(String),
    AddValue(String),
    AddNode,
    None,
}

fn are_any_changes_required(mod_ops: &ModOpsStructure) -> bool {
    let are_changes_required_for_children = mod_ops
        .children
        .iter()
        .any(|child| are_any_changes_required(child));

    match mod_ops.kind {
        ModOpsKind::ReplaceValue(_) | ModOpsKind::AddValue(_) | ModOpsKind::AddNode => true,
        ModOpsKind::None => are_changes_required_for_children,
    }
}

fn create_mod_ops_structure(content: &XmlNode, state: &State) -> ModOpsStructure {
    let (kind, mod_ops) = match &content.data {
        XmlNodeData::Branch(children) => {
            let child_mod_ops = children
                .iter()
                .map(|child| create_mod_ops_structure(&child, state))
                .filter(|mod_ops| are_any_changes_required(&mod_ops))
                .collect::<Vec<_>>();
            match content.present {
                true => (ModOpsKind::None, child_mod_ops),
                false => match child_mod_ops.is_empty() {
                    true => (ModOpsKind::None, Vec::new()),
                    false => (ModOpsKind::AddNode, child_mod_ops),
                },
            }
        }
        XmlNodeData::Leaf(old_value) => match (state, content.present) {
            (State::Included, true) => (
                ModOpsKind::ReplaceValue(new_value(&content.name, old_value)),
                Vec::new(),
            ),
            (State::Included, false) => (ModOpsKind::None, Vec::new()),
            (State::Excluded, true) => (ModOpsKind::None, Vec::new()),
            (State::Excluded, false) => (ModOpsKind::AddValue(old_value.clone()), Vec::new()),
            (State::ExcludedByAncestor, true) => (ModOpsKind::None, Vec::new()),
            (State::ExcludedByAncestor, false) => (ModOpsKind::None, Vec::new()),
            (State::Forced, true) => (
                ModOpsKind::ReplaceValue(new_value(&content.name, old_value)),
                Vec::new(),
            ),
            (State::Forced, false) => (
                ModOpsKind::AddValue(new_value(&content.name, old_value)),
                Vec::new(),
            ),
            (State::ForcedByAncestor, true) => (
                ModOpsKind::ReplaceValue(new_value(&content.name, old_value)),
                Vec::new(),
            ),
            (State::ForcedByAncestor, false) => (ModOpsKind::None, Vec::new()),
        },
        XmlNodeData::None => (ModOpsKind::None, Vec::new()),
    };

    let mod_ops_structure = ModOpsStructure {
        name: content.name.clone(),
        kind,
        children: mod_ops,
    };

    mod_ops_structure
}

fn new_value(name: &str, current_value: &str) -> String {
    match name {
        "TransporterSpeed" => {
            let mut value: f64 = current_value.parse().unwrap();
            value = value * 30.0;
            value.to_string()
        }
        "ResolverMovementSpeed" | "IntensityDecreaseRate" => {
            let mut value: f64 = current_value.parse().unwrap();
            value = value * 10.0;
            value.to_string()
        }
        "StorageAmount" => {
            let mut value: usize = current_value.parse().unwrap();
            value = value * 10;
            value.to_string()
        }
        "IndustrializationDistance"
        | "FullSatisfactionDistance"
        | "NoSatisfactionDistance"
        | "HeatRange" => {
            let mut value: usize = current_value.parse().unwrap();
            value = value * 2;
            value.to_string()
        }
        "CycleTime" => {
            let mut value: usize = current_value.parse().unwrap();
            value = value / 5;
            value.to_string()
        }
        "LoadingTime" | "UnloadingTime" => {
            let mut value: f64 = current_value.parse().unwrap();
            value = value / 5.0;
            value = value.ceil();
            value.to_string()
        }
        "CraftingTime" => {
            let mut value: usize = current_value.parse().unwrap();
            value = value / 10;
            value.to_string()
        }
        "MinPauseBetweenEvents" | "MaxPauseBetweenEvents" => {
            let mut value: usize = current_value.parse().unwrap();
            value = value / 5000;
            value.to_string()
        }
        "ResolverUnitCount" => {
            let mut value: usize = current_value.parse().unwrap();
            value = value + 1;
            value.to_string()
        }
        "MoveInMs" | "MoveOutMs" | "MoveRandomMs" => String::from("10"),
        _ => {
            todo!("new value {} not implemented yet", name);
        }
    }
}

fn create_mod_directory(mod_path: &PathBuf) {
    match std::fs::create_dir(&mod_path) {
        Err(error) => match error.kind() {
            std::io::ErrorKind::AlreadyExists => (),
            _ => panic!("Error creating {mod_path:?}: {error}"),
        },
        _ => (),
    }
}

pub(crate) fn delete_mod_files(mod_path: &PathBuf) {
    match std::fs::remove_dir_all(mod_path) {
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => (),
            _ => panic!("Error deleting {mod_path:?}: {error}"),
        },
        _ => (),
    }
}

fn enhanced_name(mod_name: &str) -> String {
    format!("JJ's Enhanced {}", mod_name)
}
