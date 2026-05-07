use std::{fs, io, path::Path};

use crate::config::{Config, config_for_dir, config_for_file};
use crate::formatting::format_file;
pub mod config;
pub mod constants;
pub mod formatting;
pub mod parsing;

pub fn process_path(path: &Path) -> io::Result<()> {
    let config = Config::default();

    if path.is_dir() {
        traverse_dirs(path, config)
    } else {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if !is_supported_file(&filename) {
            println!("unsupported file format");
            return Ok(());
        }

        let config = config_for_file(path, config)?;
        format_file(&filename, path, &config)
    }
}

pub fn traverse_dirs(dir: &Path, parent_config: Config) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let config = config_for_dir(dir, parent_config)?;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            traverse_dirs(&path, config.clone())?;
        } else {
            let filename = entry.file_name().to_str().unwrap_or("").to_string();

            if !is_supported_file(&filename) {
                continue;
            }

            format_file(&filename, &path, &config)?;
        }
    }

    Ok(())
}

pub fn is_supported_file(filename: &str) -> bool {
    filename.ends_with(".sql") || filename.ends_with(".py")
}
