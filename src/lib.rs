use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::config::{Config, config_for_file, config_for_walkdir_path};
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

    let mut configs = HashMap::<PathBuf, Config>::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|entry| {
            !entry
                .file_name()
                .to_str()
                .is_some_and(|filename| filename.starts_with('.'))
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if !is_supported_file(&filename) {
            continue;
        }

        let config = config_for_walkdir_path(path, dir, parent_config.clone(), &mut configs)?;

        let Some(extension) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };

        if !config
            .filetypes()
            .iter()
            .filter_map(|value| value.as_str())
            .any(|filetype| filetype == "*" || filetype == extension)
        {
            continue;
        }

        format_file(&filename, path, &config)?;
    }

    Ok(())
}

pub fn is_supported_file(filename: &str) -> bool {
    filename.ends_with(".sql")
        || filename.ends_with(".py")
        || filename.ends_with(".rs")
        || filename.ends_with(".js")
        || filename.ends_with(".ts")
        || filename.ends_with(".mjs")
        || filename.ends_with(".vue")
}
