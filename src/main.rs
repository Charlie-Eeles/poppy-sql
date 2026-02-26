use std::{
    env,
    fs::{self},
    io,
    path::{Path, PathBuf},
};

use clap::Parser;
use sqlformat::{Dialect, FormatOptions, QueryParams, format};

const IGNORE_STRING: &str = "--poppy-ignore";

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    file: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let arg = Args::parse();

    if let Some(path) = arg.file {
        let filename = String::from(path.file_name().unwrap().to_str().unwrap_or(""));

        if !filename.ends_with(".sql") && !filename.ends_with(".py") {
            println!("unsupported file format");
            return Ok(());
        }

        format_file(filename, path)?;
        return Ok(());
    }

    let current_dir = env::current_dir().unwrap();
    let dir = Path::new(&current_dir);
    traverse_dirs(dir)
}

fn traverse_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                traverse_dirs(&path)?;
            } else {
                let filename = String::from(entry.file_name().to_str().unwrap_or(""));
                if !filename.ends_with(".sql") && !filename.ends_with(".py") {
                    continue;
                }

                format_file(filename, path)?;
            }
        }
    }
    Ok(())
}

fn format_file(filename: String, path: PathBuf) -> io::Result<()> {
    println!("{}", filename);

    if filename.ends_with(".sql") {
        let contents = fs::read_to_string(&path).unwrap_or_default();

        if contents.contains(IGNORE_STRING) {
            return Ok(());
        }

        let mut new_contents = format_sql(&contents);
        new_contents.push('\n');

        if new_contents != contents {
            println!("Changes applied to: {}", filename);
            fs::write(&path, new_contents)?;
        }
    }

    if filename.ends_with(".py") {
        let contents = fs::read_to_string(&path).unwrap_or_default();
        let new_contents = format_sql_in_python_file(&contents);

        if new_contents != contents {
            println!("Changes applied to: {}", filename);
            fs::write(&path, new_contents)?;
        }
    }

    Ok(())
}

fn format_sql_in_python_file(contents: &str) -> String {
    let mut output = String::with_capacity(contents.len());
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
            return output;
        };

        let (raw_sql, after_sql) = unprocessed_contents.split_at(end_rel);
        let do_format =
            !is_fstring && raw_sql.trim_end().ends_with(';') && !raw_sql.contains(IGNORE_STRING);

        output.push_str(r#"""""#);

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
    output
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
