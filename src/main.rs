use std::path::PathBuf;

mod helper;

fn main() {
    // The path to the directory containing the properties, templates, and assets files.
    let path = PathBuf::from("H:\\")
        .join("Anno1800ModSupport")
        .join("filtered_data");

    // Get the paths of properties, templates, and assets files.
    let (properties_paths, template_paths, assets_paths) = helper::get_paths(&path);

    // Print the paths of the properties files.
    println!("Properties paths:");
    for path in properties_paths {
        println!("- {}", path.display());
    }

    // Print the paths of the templates files.
    println!("Templates paths:");
    for path in template_paths {
        println!("- {}", path.display());
    }

    // Print the paths of the assets files.
    println!("Assets paths:");
    for path in assets_paths {
        println!("- {}", path.display());
    }
}
