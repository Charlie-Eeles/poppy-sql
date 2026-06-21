use colored::Colorize;
use sqlx::postgres::{PgDatabaseError, PgErrorPosition};
use sqlx::{PgPool, query_scalar};

pub async fn validate_query(pool: &PgPool, query_string: String) -> bool {
    let sql = format!("EXPLAIN {query_string}");

    match query_scalar::<_, String>(&sql).fetch_all(pool).await {
        Ok(rows) => {
            println!("{}", "✔️ Query valid".green());

            if let Some(first_line) = rows.first() {
                println!("{} {}", "Plan:".cyan(), first_line.trim());
            }

            true
        }
        Err(err) => {
            let Some(db_err) = err.as_database_error() else {
                println!("{}", err.to_string());
                return false;
            };

            let Some(pg_err) = db_err.try_downcast_ref::<PgDatabaseError>() else {
                println!("{}", db_err.message().to_string());
                return false;
            };

            let Some(PgErrorPosition::Original(pos)) = pg_err.position() else {
                println!("{}", db_err.message().to_string());
                return false;
            };

            let line = sql[..pos as usize - 1].lines().count();

            println!("{} {}", "Error:".red(), db_err.message());
            println!("{} Error likely at or near line {line}", "Hint:".yellow());

            false
        }
    }
}
