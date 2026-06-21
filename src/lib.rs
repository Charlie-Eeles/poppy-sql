use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use walkdir::{DirEntry, WalkDir};

use crate::{
    config::{Config, config_for_file, config_for_walkdir_path},
    constants::{SKIPPED_DIRS, SUPPORTED_EXTENSIONS},
    formatting::formatting::format_file,
};

pub mod config;
pub mod constants;
pub mod formatting;
pub mod parsing;
pub mod watch;

fn should_walk_entry(entry: &DirEntry) -> bool {
    let Some(filename) = entry.file_name().to_str() else {
        return false;
    };

    if filename.starts_with('.') && filename != "." {
        return false;
    }

    if entry.file_type().is_dir() && SKIPPED_DIRS.contains(&filename) {
        return false;
    }

    true
}

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

        if !is_supported_file(path) {
            println!("Unsupported file format");
            return Ok(());
        }

        let Some(extension) = path.extension().and_then(|s| s.to_str()) else {
            return Ok(());
        };

        if !config
            .filetypes()
            .iter()
            .filter_map(|value| value.as_str())
            .any(|filetype| filetype == "*" || filetype == extension)
        {
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
        .filter_entry(should_walk_entry)
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if !path.is_file() || !is_supported_file(path) {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

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

pub fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|extension| SUPPORTED_EXTENSIONS.contains(&extension))
}
