use colored::Colorize;
use sqlx::postgres::{PgDatabaseError, PgErrorPosition};
use sqlx::{PgPool, query, query_scalar};

pub async fn validate_query(pool: &PgPool, query_string: String, query_number: usize) -> bool {
    let result = if is_explainable(&query_string) {
        query_scalar::<_, String>(&format!("EXPLAIN {query_string}"))
            .fetch_all(pool)
            .await
            .map(|rows| rows.first().cloned())
            .map_err(|err| (err, "EXPLAIN ".len()))
    } else {
        match pool.begin().await {
            Ok(mut tx) => {
                let result = query(&query_string).execute(&mut *tx).await;
                let _ = tx.rollback().await;

                result.map(|_| None).map_err(|err| (err, 0))
            }
            Err(err) => Err((err, 0)),
        }
    };

    match result {
        Ok(plan) => {
            println!(
                "{} {} {}",
                "✔️ Query valid".green(),
                format!("({query_number})").dimmed(),
                truncate_query(query_string).dimmed()
            );

            if let Some(plan) = plan {
                println!("{} {}", "Plan:".cyan(), plan.trim());
            }

            true
        }
        Err((err, offset)) => {
            let Some(db_err) = err.as_database_error() else {
                println!("{} {}", format!("Query {query_number}:").red(), err);
                println!("{} {}", "Query:".yellow(), query_string.dimmed());
                return false;
            };

            let error_line = db_err
                .try_downcast_ref::<PgDatabaseError>()
                .and_then(|err| match err.position() {
                    Some(PgErrorPosition::Original(pos)) => {
                        let query_pos = (pos as usize).saturating_sub(offset + 1);
                        Some(
                            query_string[..query_pos.min(query_string.len())]
                                .lines()
                                .count(),
                        )
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

fn is_explainable(sql: &str) -> bool {
    let first_word = sql
        .trim_start()
        .split(|c: char| c.is_whitespace() || c == '(')
        .next()
        .unwrap_or("")
        .to_ascii_uppercase();

    matches!(
        first_word.as_str(),
        "SELECT" | "INSERT" | "UPDATE" | "DELETE" | "MERGE" | "WITH" | "VALUES" | "TABLE"
    )
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
