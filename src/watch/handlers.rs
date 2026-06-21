use colored::Colorize;
use sqlx::postgres::{PgDatabaseError, PgErrorPosition};
use sqlx::{PgPool, query_scalar};

pub async fn validate_query(pool: &PgPool, query_string: String, query_number: usize) -> bool {
    let sql = format!("EXPLAIN {query_string}");

    match query_scalar::<_, String>(&sql).fetch_all(pool).await {
        Ok(rows) => {
            println!(
                "{} {} {}",
                "✔️ Query valid".green(),
                format!("({query_number})").dimmed(),
                truncate_query(query_string).dimmed()
            );

            if let Some(first_line) = rows.first() {
                println!("{} {}", "Plan:".cyan(), first_line.trim());
            }

            true
        }
        Err(err) => {
            let Some(db_err) = err.as_database_error() else {
                println!("{} {}", format!("Query {query_number}:").red(), err);
                println!("{} {}", "Query:".yellow(), query_string.dimmed());
                return false;
            };

            let error_line = db_err
                .try_downcast_ref::<PgDatabaseError>()
                .and_then(|err| match err.position() {
                    Some(PgErrorPosition::Original(pos)) => {
                        let query_pos = (pos as usize).saturating_sub("EXPLAIN ".len() + 1);
                        Some(query_string[..query_pos].lines().count())
                    }
                    _ => None,
                });

            println!(
                "{} {}",
                format!("Error in query {query_number}:").red(),
                db_err.message()
            );
            println!("{}", "Query:".yellow());

            for (index, line) in query_string.lines().enumerate() {
                let line_number = index + 1;

                if Some(line_number) == error_line {
                    println!(
                        "{} {} {}",
                        "->".red(),
                        format!("{line_number}:").red(),
                        line.dimmed()
                    );
                } else {
                    println!(
                        "{} {} {}",
                        "  ".dimmed(),
                        format!("{line_number}:").dimmed(),
                        line.dimmed()
                    );
                }
            }

            false
        }
    }
}

fn truncate_query(sql: String) -> String {
    match pg_query::parse(sql.as_str()) {
        Ok(query) => match query.truncate(20) {
            Ok(truncated) => truncated,
            Err(err) => err.to_string(),
        },
        Err(err) => err.to_string(),
    }
}
