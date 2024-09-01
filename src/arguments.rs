use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "Anno1800ModCreator")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A tool to create mods for Anno 1800")]
pub(crate) struct Arguments {
    #[arg(value_parser = check_if_path_exists)]
    pub(crate) input_path: PathBuf,

    #[arg(value_parser = check_if_path_exists)]
    pub(crate) output_path: PathBuf,
}

fn check_if_path_exists(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Path does not exist: {}", path.display()))
    }
}
