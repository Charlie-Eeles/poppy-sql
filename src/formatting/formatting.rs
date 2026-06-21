use std::{fs, io, path::Path};

use crate::Config;
use crate::constants::IGNORE_STRING;
use crate::formatting::entry::{Dialect, FormatOptions, QueryParams, format};
use crate::parsing::javascript::find_sql_in_javascript_file;
use crate::parsing::python::find_sql_in_python_file;
use crate::parsing::rust::find_sql_in_rust_file;

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

    if filename.ends_with(".rs") {
        let contents = fs::read_to_string(path).unwrap_or_default();
        let result = find_sql_in_rust_file(&contents, true, config);
        let new_contents = result.content;

        if new_contents != contents {
            println!("Changes applied to: {filename}");
            fs::write(path, new_contents)?;
        }
    }

    if filename.ends_with(".js")
        || filename.ends_with(".ts")
        || filename.ends_with(".mjs")
        || filename.ends_with(".vue")
    {
        let contents = fs::read_to_string(path).unwrap_or_default();
        let result = find_sql_in_javascript_file(&contents, true, config);
        let new_contents = result.content;

        if new_contents != contents {
            println!("Changes applied to: {filename}");
            fs::write(path, new_contents)?;
        }
    }

    Ok(())
}

pub fn format_sql(sql: &str, config: &Config) -> String {
    let format_config = &config.format;

    format(
        sql,
        &QueryParams::None,
        &FormatOptions {
            indent: super::entry::Indent::Spaces(format_config.indent.unwrap_or(4)),
            uppercase: format_config.uppercase,
            joins_as_top_level: format_config.joins_as_top_level.unwrap_or(true),
            dialect: Dialect::PostgreSql,
            lines_between_queries: format_config.lines_between_queries.unwrap_or(2),
            add_semicolons: format_config.add_semicolons.unwrap_or(false),
            ..Default::default()
        },
    )
}
