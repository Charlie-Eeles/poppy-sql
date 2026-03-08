use std::{
    fs,
    io,
    path::Path,
};

use sqlformat::{format, Dialect, FormatOptions, QueryParams};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser as SqlParser;

pub const IGNORE_STRING: &str = "--poppy-ignore";

pub struct PythonSqlResult {
    pub content: String,
    pub queries: Vec<String>,
}

pub fn process_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        traverse_dirs(path)
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

        format_file(&filename, path)
    }
}

pub fn traverse_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                traverse_dirs(&path)?;
            } else {
                let filename = entry.file_name().to_str().unwrap_or("").to_string();

                if !is_supported_file(&filename) {
                    continue;
                }

                format_file(&filename, &path)?;
            }
        }
    }

    Ok(())
}

pub fn format_file(filename: &str, path: &Path) -> io::Result<()> {
    println!("{filename}");

    if filename.ends_with(".sql") {
        let contents = fs::read_to_string(path).unwrap_or_default();

        if contents.contains(IGNORE_STRING) {
            return Ok(());
        }

        let mut new_contents = format_sql(&contents);
        new_contents.push('\n');

        if new_contents != contents {
            println!("Changes applied to: {filename}");
            fs::write(path, new_contents)?;
        }
    }

    if filename.ends_with(".py") {
        let contents = fs::read_to_string(path).unwrap_or_default();
        let result = find_sql_in_python_file(&contents, true);
        let new_contents = result.content;

        if new_contents != contents {
            println!("Changes applied to: {filename}");
            fs::write(path, new_contents)?;
        }
    }

    Ok(())
}

pub fn find_sql_in_python_file(contents: &str, format_file_content: bool) -> PythonSqlResult {
    let mut output = String::with_capacity(contents.len());
    let mut queries = Vec::new();
    let mut unprocessed_contents = contents;
    let dialect = PostgreSqlDialect {};

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

        let is_valid_sql_query = !is_fstring
            && SqlParser::parse_sql(&dialect, raw_sql).is_ok();

        let do_format = format_file_content
            && is_valid_sql_query
            && !raw_sql.contains(IGNORE_STRING);

        output.push_str(r#"""""#);

        if is_valid_sql_query {
            queries.push(raw_sql.to_string());
        }

        if do_format {
            let formatted = format_sql(raw_sql);

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

pub fn format_sql(sql: &str) -> String {
    format(
        sql,
        &QueryParams::None,
        &FormatOptions {
            indent: sqlformat::Indent::Spaces(4),
            uppercase: Some(true),
            joins_as_top_level: true,
            dialect: Dialect::PostgreSql,
            lines_between_queries: 2,
            ..Default::default()
        },
    )
}

pub fn is_supported_file(filename: &str) -> bool {
    filename.ends_with(".sql") || filename.ends_with(".py")
}
