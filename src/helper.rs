use std::path::PathBuf;
use walkdir::WalkDir;

pub fn get_paths(path: &PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
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
