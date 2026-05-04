use std::{fs, io, path::Path};

use serde::Deserialize;
use sqlformat::{Dialect, FormatOptions, QueryParams, format};
use sqlparser::dialect::{GenericDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser as SqlParser;

pub const IGNORE_STRING: &str = "--poppy-ignore";
pub const CONFIG_FILE: &str = ".poppy.toml";

const DEFAULT_CONFIG: &str = include_str!("./default.poppy.toml");

pub struct PythonSqlResult {
    pub content: String,
    pub queries: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub dialect: String,
    pub format: FormatConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormatConfig {
    pub indent: Option<u8>,
    pub uppercase: Option<bool>,
    pub joins_as_top_level: Option<bool>,
    pub lines_between_queries: Option<u8>,
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
        }
    }
}

impl Config {
    fn merge(self, override_config: Config) -> Self {
        Self {
            dialect: self.dialect,
            format: self.format.merge(override_config.format),
        }
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
        }
    }
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

fn config_for_file(path: &Path, fallback: Config) -> io::Result<Config> {
    let Some(dir) = path.parent() else {
        return Ok(fallback);
    };

    config_for_ancestors(dir, fallback)
}

fn config_for_dir(dir: &Path, parent_config: Config) -> io::Result<Config> {
    let config_path = dir.join(CONFIG_FILE);

    if !config_path.exists() {
        return Ok(parent_config);
    }

    let contents = fs::read_to_string(config_path)?;
    let local_config = parse_config(&contents)?;

    Ok(parent_config.merge(local_config))
}

fn config_for_ancestors(start_dir: &Path, fallback: Config) -> io::Result<Config> {
    let mut dirs = start_dir.ancestors().collect::<Vec<_>>();
    dirs.reverse();

    let mut config = fallback;

    for dir in dirs {
        let config_path = dir.join(CONFIG_FILE);

        if config_path.exists() {
            let contents = fs::read_to_string(config_path)?;
            let local_config = parse_config(&contents)?;
            config = config.merge(local_config);
        }
    }

    Ok(config)
}

fn parse_config(config: &str) -> io::Result<Config> {
    toml::from_str(config).map_err(invalid_config)
}

fn invalid_config(error: toml::de::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid {CONFIG_FILE}: {error}"),
    )
}

pub fn format_file(filename: &str, path: &Path, config: &Config) -> io::Result<()> {
    println!("{filename}");

    if filename.ends_with(".sql") {
        let contents = fs::read_to_string(path).unwrap_or_default();

        if contents.contains(IGNORE_STRING) {
            return Ok(());
        }

        let mut new_contents = format_sql(&contents, config);
        new_contents.push('\n');

        if new_contents != contents {
            println!("Changes applied to: {filename}");
            fs::write(path, new_contents)?;
        }
    }

    if filename.ends_with(".py") {
        let contents = fs::read_to_string(path).unwrap_or_default();
        let result = find_sql_in_python_file(&contents, true, config);
        let new_contents = result.content;

        if new_contents != contents {
            println!("Changes applied to: {filename}");
            fs::write(path, new_contents)?;
        }
    }

    Ok(())
}

pub fn find_sql_in_python_file(
    contents: &str,
    format_file_content: bool,
    config: &Config,
) -> PythonSqlResult {
    let mut output = String::with_capacity(contents.len());
    let mut queries = Vec::new();
    let mut unprocessed_contents = contents;

    while let Some(start) = unprocessed_contents.find(r#"""""#) {
        let is_fstring =
            start > 0 && matches!(unprocessed_contents.as_bytes()[start - 1], b'f' | b'F');

        let (prefix, after_prefix) = unprocessed_contents.split_at(start);
        output.push_str(prefix);

        let indent: String = prefix
            .lines()
            .next_back()
            .unwrap_or("")
            .chars()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .collect();

        unprocessed_contents = &after_prefix[3..];

        let Some(end_rel) = unprocessed_contents.find(r#"""""#) else {
            output.push_str(r#"""""#);
            output.push_str(unprocessed_contents);
            return PythonSqlResult {
                content: output,
                queries,
            };
        };

        let (raw_sql, after_sql) = unprocessed_contents.split_at(end_rel);

        let parsed_sql = match config.dialect.as_str() {
            "PostgreSQL" => SqlParser::parse_sql(&PostgreSqlDialect {}, raw_sql),
            "MySQL" => SqlParser::parse_sql(&MySqlDialect {}, raw_sql),
            "SQLite" => SqlParser::parse_sql(&SQLiteDialect {}, raw_sql),
            _ => SqlParser::parse_sql(&GenericDialect {}, raw_sql),
        };

        let is_valid_sql_query = !is_fstring && parsed_sql.is_ok();

        let do_format =
            format_file_content && is_valid_sql_query && !raw_sql.contains(IGNORE_STRING);

        output.push_str(r#"""""#);

        if is_valid_sql_query {
            queries.push(raw_sql.to_string());
        }

        if do_format {
            let formatted = format_sql(raw_sql, config);

            output.push('\n');

            for line in formatted.lines() {
                output.push_str(&indent);
                output.push_str(line);
                output.push('\n');
            }

            output.push_str(&indent);
        } else {
            output.push_str(raw_sql);
        }

        output.push_str(r#"""""#);
        unprocessed_contents = &after_sql[3..];
    }

    output.push_str(unprocessed_contents);

    PythonSqlResult {
        content: output,
        queries,
    }
}

fn format_sql(sql: &str, config: &Config) -> String {
    let format_config = &config.format;

    format(
        sql,
        &QueryParams::None,
        &FormatOptions {
            indent: sqlformat::Indent::Spaces(format_config.indent.unwrap_or(4)),
            uppercase: format_config.uppercase,
            joins_as_top_level: format_config.joins_as_top_level.unwrap_or(true),
            dialect: match config.dialect.as_str() {
                "PostgreSQL" => Dialect::PostgreSql,
                _ => Dialect::Generic,
            },
            lines_between_queries: format_config.lines_between_queries.unwrap_or(2),
            ..Default::default()
        },
    )
}

pub fn is_supported_file(filename: &str) -> bool {
    filename.ends_with(".sql") || filename.ends_with(".py")
}
