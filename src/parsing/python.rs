use sqlparser::dialect::{GenericDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser as SqlParser;

use crate::Config;
use crate::constants::IGNORE_STRING;
use crate::formatting::format_sql;
use crate::parsing::common::ParsedSqlResult;

pub fn find_sql_in_python_file(
    contents: &str,
    format_file_content: bool,
    config: &Config,
) -> ParsedSqlResult {
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
            return ParsedSqlResult {
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

    ParsedSqlResult {
        content: output,
        queries,
    }
}
