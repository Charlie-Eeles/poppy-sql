use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use toml::value::Array;

use crate::constants::{CONFIG_FILE, DEFAULT_CONFIG};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub filetypes: Option<Array>,

    #[serde(default)]
    pub format: FormatConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormatConfig {
    pub indent: Option<u8>,
    pub uppercase: Option<bool>,
    pub joins_as_top_level: Option<bool>,
    pub lines_between_queries: Option<u8>,
    pub add_semicolons: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        parse_config(DEFAULT_CONFIG).expect("default config should be valid")
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent: Some(4),
            uppercase: Some(true),
            joins_as_top_level: Some(true),
            lines_between_queries: Some(2),
            add_semicolons: Some(false),
        }
    }
}

impl Config {
    fn merge(self, override_config: Config) -> Self {
        Self {
            filetypes: override_config.filetypes.or(self.filetypes),
            format: self.format.merge(override_config.format),
        }
    }

    pub fn filetypes(&self) -> &Array {
        self.filetypes
            .as_ref()
            .expect("filetypes should be set after merging with default config")
    }
}

impl FormatConfig {
    fn merge(self, override_config: FormatConfig) -> Self {
        Self {
            indent: override_config.indent.or(self.indent),
            uppercase: override_config.uppercase.or(self.uppercase),
            joins_as_top_level: override_config
                .joins_as_top_level
                .or(self.joins_as_top_level),
            lines_between_queries: override_config
                .lines_between_queries
                .or(self.lines_between_queries),
            add_semicolons: override_config.add_semicolons.or(self.add_semicolons),
        }
    }
}

pub fn config_for_file(path: &Path, fallback: Config) -> io::Result<Config> {
    let Some(dir) = path.parent() else {
        return Ok(fallback);
    };

    config_for_ancestors(dir, fallback)
}

pub fn config_for_dir(dir: &Path, parent_config: Config) -> io::Result<Config> {
    let config_path = dir.join(CONFIG_FILE);

    if !config_path.exists() {
        return Ok(parent_config);
    }

    let contents = fs::read_to_string(config_path)?;
    let local_config = parse_config(&contents)?;

    Ok(parent_config.merge(local_config))
}

pub fn config_for_ancestors(start_dir: &Path, fallback: Config) -> io::Result<Config> {
    let mut dirs = start_dir.ancestors().collect::<Vec<_>>();
    dirs.reverse();

    let mut config = fallback;

    for dir in dirs {
        config = config_for_dir(dir, config)?;
    }

    Ok(config)
}

pub fn config_for_walkdir_path(
    path: &Path,
    root: &Path,
    fallback: Config,
    configs: &mut HashMap<PathBuf, Config>,
) -> io::Result<Config> {
    let parent = path.parent().unwrap_or(root);

    if let Some(config) = configs.get(parent) {
        return Ok(config.clone());
    }

    let relative_parent = parent.strip_prefix(root).unwrap_or(parent);
    let mut current = root.to_path_buf();

    let mut config = configs
        .get(root)
        .cloned()
        .unwrap_or_else(|| fallback.clone());

    config = config_for_dir(&current, config)?;

    for component in relative_parent.components() {
        current.push(component);

        if let Some(cached_config) = configs.get(&current) {
            config = cached_config.clone();
            continue;
        }

        config = config_for_dir(&current, config)?;
        configs.insert(current.clone(), config.clone());
    }

    Ok(config)
}

pub fn parse_config(config: &str) -> io::Result<Config> {
    toml::from_str(config).map_err(invalid_config)
}

pub fn invalid_config(error: toml::de::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid {CONFIG_FILE}: {error}"),
    )
}
