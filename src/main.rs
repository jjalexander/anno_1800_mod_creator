use identifier::{Identifier, Kind, ParentIdentifier};
use itertools::Itertools;
use state::State;
use std::{collections::HashMap, path::PathBuf};
use xml_node::XmlNode;
use xml_structure::{Content, XmlTag};

mod helper;
mod identifier;
mod state;
mod xml_node;
mod xml_structure;

fn main() {
    // The path to the directory containing the properties, templates, and assets files.
    let path = PathBuf::from("H:\\")
        .join("Anno1800ModSupport")
        .join("filtered_data");

    // The path to the directory containing the mod files.
    let mod_path = PathBuf::from("D:\\")
        .join("Ubisoft Games")
        .join("Anno 1800")
        .join("mods");

    // Get the paths of properties, templates, and assets files.
    let (properties_paths, template_paths, assets_paths) = helper::get_paths(&path);

    // Print the paths of the properties files.
    println!("Properties paths:");
    for path in &properties_paths {
        println!("- {}", path.display());
    }
    println!("----------------------------------------");

    // Print the paths of the templates files.
    println!("Templates paths:");
    for path in &template_paths {
        println!("- {}", path.display());
    }
    println!("----------------------------------------");

    // Print the paths of the assets files.
    println!("Assets paths:");
    for path in &assets_paths {
        println!("- {}", path.display());
    }
    println!("----------------------------------------");

    // Mod name
    let mod_name: &str = "Production";

    // XML structure
    let query: XmlTag = XmlTag {
        name: "FactoryBase".to_string(),
        content: Content::Branch(vec![XmlTag {
            name: "CycleTime".to_string(),
            content: Content::Leaf,
        }]),
    };

    // Excluded templates
    let excluded_templates: Vec<String> = vec![
        "Heater_Arctic".to_owned(),
        "PowerplantBuilding".to_owned(),
        "BuffFactoryModule".to_owned(),
        "Mall".to_owned(),
        "TowerRestaurant".to_owned(),
    ];

    // Excluded GUIDs
    let excluded_guids: Vec<String> = vec![];

    // Forced GUIDs
    let forced_guids: Vec<String> = vec!["24861".to_owned(), "24845".to_owned()];

    // // Mod name
    // let mod_name = "Crafting";

    // // Xml structure
    // let query = XmlTag {
    //     name: "Craftable".to_string(),
    //     content: Content::Branch(vec![XmlTag {
    //         name: "CraftingTime".to_string(),
    //         content: Content::Leaf,
    //     }]),
    // };

    // // Excluded templates
    // let excluded_templates: Vec<String> = vec![];

    // // Excluded GUIDs
    // let excluded_guids: Vec<String> = vec![];

    // // Forced GUIDs
    // let forced_guids: Vec<String> = vec![];

    let mut identifiers: Vec<Identifier> = Vec::new();
    let mut identifiers_as_parent: HashMap<ParentIdentifier, Identifier> = HashMap::new();
    let mut parent_identifiers: HashMap<Identifier, ParentIdentifier> = HashMap::new();
    let mut states: HashMap<Identifier, State> = HashMap::new();
    let mut contents: HashMap<Identifier, XmlNode> = HashMap::new();

    // Iterate over the properties files.
    for path in properties_paths {
        // read the xml file
        let xml_string = std::fs::read_to_string(&path).unwrap();

        // parse the xml file
        let xml = roxmltree::Document::parse(&xml_string).unwrap();

        let inner_data_path = path
            .iter()
            .skip(5)
            .map(|s| s.to_str().unwrap())
            .collect::<Vec<_>>()
            .join("\\");

        xml.descendants()
            .filter(|node| node.tag_name().name() == "DefaultValues")
            .for_each(|node| {
                let identifier = create_default_values_identifier(&inner_data_path, &node);

                if !helper::has_direct_child(&node, &query) {
                    return;
                }

                let Some(content) = helper::extract_content(&node, &query, None) else {
                    return;
                };

                identifiers.push(identifier.clone());
                identifiers_as_parent.insert(ParentIdentifier::DefaultValues, identifier.clone());
                parent_identifiers.insert(identifier.clone(), ParentIdentifier::None);
                states.insert(identifier.clone(), State::Included);
                contents.insert(identifier.clone(), content);
            });
    }

    println!("Total default values nodes: {}", identifiers.len());

    // Iterate over the templates files.
    for path in template_paths {
        // read the xml file
        let xml_string = std::fs::read_to_string(&path).unwrap();

        // parse the xml file
        let xml = roxmltree::Document::parse(&xml_string).unwrap();

        let inner_data_path = path
            .iter()
            .skip(5)
            .map(|s| s.to_str().unwrap())
            .collect::<Vec<_>>()
            .join("\\");

        xml.descendants()
            .filter(|node| node.tag_name().name() == "Template")
            .for_each(|node| {
                let Some(identifier) = create_template_identifier(&inner_data_path, &node) else {
                    return;
                };

                if !helper::has_properties_child(&node, &query) {
                    return;
                }

                let Some(content) = helper::extract_content_from_properties(
                    &node,
                    &query,
                    contents.get(&identifiers_as_parent[&ParentIdentifier::DefaultValues]),
                ) else {
                    return;
                };

                identifiers.push(identifier.clone());
                identifiers_as_parent.insert(
                    ParentIdentifier::Template(identifier.value.clone()),
                    identifier.clone(),
                );
                parent_identifiers.insert(identifier.clone(), ParentIdentifier::DefaultValues);
                states.insert(
                    identifier.clone(),
                    match excluded_templates.contains(&identifier.value) {
                        true => State::Excluded,
                        false => State::Included,
                    },
                );
                contents.insert(identifier.clone(), content);
            });
    }

    println!(
        "Total default values and template nodes: {}",
        identifiers.len()
    );

    // Iterate over the assets files.
    for path in &assets_paths {
        // read the xml file
        let xml_string = std::fs::read_to_string(&path).unwrap();

        // parse the xml file
        let xml = roxmltree::Document::parse(&xml_string).unwrap();

        let inner_data_path = path
            .iter()
            .skip(5)
            .map(|s| s.to_str().unwrap())
            .collect::<Vec<_>>()
            .join("\\");

        xml.descendants()
            .filter(|node| node.tag_name().name() == "Asset")
            .for_each(|node| {
                let identifier = create_asset_identifier(&inner_data_path, &node);
                if identifiers.contains(&identifier) {
                    return;
                }

                let node_parent_identifier = match create_asset_parent_identifier(&node) {
                    ParentIdentifier::Template(name) => ParentIdentifier::Template(name),
                    ParentIdentifier::GUID(guid) => ParentIdentifier::GUID(guid),
                    _ => return,
                };

                let Some(parent_identifier) = identifiers_as_parent.get(&node_parent_identifier)
                else {
                    return;
                };
                let state = match forced_guids.contains(&identifier.value) {
                    true => State::Forced,
                    false => match excluded_guids.contains(&identifier.value) {
                        true => State::Excluded,
                        false => match states.get(parent_identifier).unwrap() {
                            State::Included => State::Included,
                            State::Excluded | State::ExcludedByAncestor => {
                                State::ExcludedByAncestor
                            }
                            State::Forced | State::ForcedByAncestor => State::ForcedByAncestor,
                        },
                    },
                };

                let Some(content) = helper::extract_content_from_values(
                    &node,
                    &query,
                    contents.get(&identifiers_as_parent[&ParentIdentifier::DefaultValues]),
                ) else {
                    return;
                };

                identifiers.push(identifier.clone());
                match identifier.kind {
                    Kind::GUID => {
                        identifiers_as_parent.insert(
                            ParentIdentifier::GUID(identifier.value.clone()),
                            identifier.clone(),
                        );
                    }
                    _ => (),
                }
                parent_identifiers.insert(identifier.clone(), node_parent_identifier.clone());
                states.insert(identifier.clone(), state);
                contents.insert(identifier.clone(), content);
            });
    }

    println!(
        "Total default values, template, and asset nodes: {}",
        identifiers.len()
    );

    println!("----------------------------------------");

    // // Print the data
    // for identifier in &identifiers {
    //     let mut strings = Vec::new();

    //     strings.push(format!("{:?}", identifier));

    //     match identifiers_as_parent
    //         .iter()
    //         .find(|(_, value)| *value == identifier)
    //     {
    //         Some((parent_identifier, _)) => strings.push(format!("{:?}", parent_identifier)),
    //         None => strings.push("None".to_string()),
    //     }

    //     strings.push(format!("{:?}", parent_identifiers.get(identifier).unwrap()));

    //     strings.push(format!("{:?}", states.get(identifier).unwrap()));

    //     strings.push(format!("{:?}", contents.get(identifier).unwrap()));

    //     println!("{}", strings.join(" | "));
    //     println!("----------------------------------------");
    // }

    helper::create_mod(
        &mod_path,
        mod_name,
        &identifiers,
        &identifiers_as_parent,
        &parent_identifiers,
        &states,
        &contents,
    );
}

fn create_asset_parent_identifier(node: &roxmltree::Node<'_, '_>) -> ParentIdentifier {
    if let Some(template_node) = node
        .children()
        .filter(|n| n.tag_name().name() == "Template")
        .at_most_one()
        .expect(format!("Multiple template nodes in {}", helper::get_xpath(&node)).as_str())
    {
        let template_name = template_node
            .text()
            .expect(format!("Problem with template text in {}", helper::get_xpath(&node)).as_str());
        return ParentIdentifier::Template(template_name.to_string());
    }

    if let Some(base_asset_guid_node) = node
        .children()
        .filter(|n| n.tag_name().name() == "BaseAssetGUID")
        .at_most_one()
        .expect(
            format!(
                "Multiple base asset guid nodes in {}",
                helper::get_xpath(&node)
            )
            .as_str(),
        )
    {
        let base_asset_guid = base_asset_guid_node.text().expect(
            format!(
                "Problem with base asset guid text in {}",
                helper::get_xpath(&node)
            )
            .as_str(),
        );
        return ParentIdentifier::GUID(base_asset_guid.to_string());
    }

    if let Some(scenario_base_asset_guid_node) = node
        .children()
        .filter(|n| n.tag_name().name() == "ScenarioBaseAssetGUID")
        .at_most_one()
        .expect(
            format!(
                "Multiple scenario base asset guid nodes in {}",
                helper::get_xpath(&node)
            )
            .as_str(),
        )
    {
        let scenario_base_asset_guid = scenario_base_asset_guid_node.text().expect(
            format!(
                "Problem with scenario base asset guid text in {}",
                helper::get_xpath(&node)
            )
            .as_str(),
        );
        return ParentIdentifier::GUID(scenario_base_asset_guid.to_string());
    }

    ParentIdentifier::None
}

fn create_asset_identifier(path: &String, node: &roxmltree::Node<'_, '_>) -> Identifier {
    let xpath_identifier = Identifier {
        file_path: path.clone(),
        kind: Kind::XPath,
        value: helper::get_xpath(&node),
    };

    let Some(values_node) = node
        .children()
        .filter(|n| n.tag_name().name() == "Values")
        .at_most_one()
        .expect(format!("Multiple values nodes in {}", helper::get_xpath(&node)).as_str())
    else {
        return xpath_identifier;
    };
    let Some(standard_node) = values_node
        .children()
        .filter(|n| n.tag_name().name() == "Standard")
        .at_most_one()
        .expect(format!("Multiple standard nodes in {}", helper::get_xpath(&node)).as_str())
    else {
        return xpath_identifier;
    };
    let Some(guid_node) = standard_node
        .children()
        .filter(|n| n.tag_name().name() == "GUID")
        .at_most_one()
        .expect(format!("Multiple guid nodes in {}", helper::get_xpath(&node)).as_str())
    else {
        return xpath_identifier;
    };

    let Some(guid_value) = guid_node.text() else {
        return xpath_identifier;
    };

    Identifier {
        file_path: path.clone(),
        kind: Kind::GUID,
        value: guid_value.to_string(),
    }
}

fn create_template_identifier(path: &String, node: &roxmltree::Node<'_, '_>) -> Option<Identifier> {
    let name_node = node
        .children()
        .filter(|n| n.tag_name().name() == "Name")
        .at_most_one()
        .expect(format!("Problem with name node in {}", helper::get_xpath(&node)).as_str())?;
    let name_value = name_node
        .text()
        .expect(format!("Problem with name text in {}", helper::get_xpath(&node)).as_str());

    Some(Identifier {
        file_path: path.clone(),
        kind: Kind::Name,
        value: name_value.to_string(),
    })
}

fn create_default_values_identifier(path: &String, node: &roxmltree::Node<'_, '_>) -> Identifier {
    Identifier {
        file_path: path.clone(),
        kind: Kind::XPath,
        value: helper::get_xpath(&node),
    }
}
