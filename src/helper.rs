use rayon::iter::{ParallelBridge, ParallelIterator};
use std::path::PathBuf;
use walkdir::WalkDir;

pub(crate) fn get_paths(path: &std::path::PathBuf) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut all_paths = WalkDir::new(path)
        .into_iter()
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .fold(
            || (Vec::new(), Vec::new(), Vec::new()),
            |mut acc, entry| {
                match entry.file_name().to_str() {
                    Some("properties.xml") => acc.0.push(entry.path().to_path_buf()),
                    Some("templates.xml") => acc.1.push(entry.path().to_path_buf()),
                    Some("assets.xml") => acc.2.push(entry.path().to_path_buf()),
                    _ => {}
                }
                acc
            },
        )
        .reduce(
            || (Vec::new(), Vec::new(), Vec::new()),
            |(mut all_properties_paths, mut all_templates_paths, mut all_assets_paths),
             (new_properties_paths, new_templates_paths, new_assets_paths)| {
                all_properties_paths.extend(new_properties_paths);
                all_templates_paths.extend(new_templates_paths);
                all_assets_paths.extend(new_assets_paths);
                (all_properties_paths, all_templates_paths, all_assets_paths)
            },
        );

    all_paths.0.sort();
    all_paths.1.sort();
    all_paths.2.sort();

    all_paths
}
