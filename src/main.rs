use identifier::Identifier;
use std::{collections::HashMap, path::PathBuf};
use xml_structure::XmlTag;

mod helper;
mod identifier;
mod xml_node;
mod xml_structure;

fn main() {
    // The path to the directory containing the properties, templates, and assets files.
    let path = PathBuf::from("H:\\")
        .join("Anno1800ModSupport")
        .join("filtered_data");

    // Get the paths of properties, templates, and assets files.
    let (properties_paths, template_paths, assets_paths) = helper::get_paths(&path);

    // Print the paths of the properties files.
    println!("Properties paths:");
    for path in &properties_paths {
        println!("- {}", path.display());
    }

    // Print the paths of the templates files.
    println!("Templates paths:");
    for path in &template_paths {
        println!("- {}", path.display());
    }

    // Print the paths of the assets files.
    println!("Assets paths:");
    for path in &assets_paths {
        println!("- {}", path.display());
    }

    // Mod name
    let mod_name = "Production";

    // Xml structure
    let xml_structure = XmlTag::Branch {
        name: "FactoryBase".to_string(),
        children: vec![XmlTag::Leaf {
            name: "CycleTime".to_string(),
        }],
    };

    let mut identifiers = Vec::new();
    let mut xml_nodes = HashMap::new();

    // Iterate over the properties files.
    for path in properties_paths {
        // read the xml file
        let xml_string = std::fs::read_to_string(&path).unwrap();

        // parse the xml file
        let xml = roxmltree::Document::parse(&xml_string).unwrap();

        xml.descendants()
            .filter(|node| node.tag_name().name() == "DefaultValues")
            .for_each(|node| {
                let Some(xml_node) = helper::extract_xml(&node, &xml_structure) else {
                    return;
                };

                let identifier = Identifier::XPath(helper::get_xpath(&node));

                identifiers.push(identifier.clone());
                xml_nodes.insert(identifier.clone(), xml_node);
            });
    }
    
    // Print the data
    for identifier in &identifiers {
        println!("{:?}", identifier);
        println!("{:?}", xml_nodes.get(identifier).unwrap());
    }
    
}
