use std::{env, fs, io, path::PathBuf};

use clap::Parser;
use colored::Colorize;
use dotenv::dotenv;
use poppy_sql::watch::handlers::validate_query;
use sqlx::postgres::PgPoolOptions;
use tokio::time::{Duration, sleep};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'f', long, conflicts_with = "watch")]
    format: bool,

    #[arg(short = 'w', long, conflicts_with = "format")]
    watch: bool,

    #[arg(short = 'd', long)]
    db_url: Option<String>,

    #[arg(short = 't', long = "target", value_name = "TARGET")]
    option_target: Vec<PathBuf>,

    #[arg(value_name = "TARGET")]
    targets: Vec<PathBuf>,
}

fn get_env_var_or_exit(name: &str) -> String {
    dotenv().ok();

    match env::var(name) {
        Ok(val) => val,
        Err(_) => {
            println!("Required variable not set in environment: {name}");
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.watch {
        let db_url = args
            .db_url
            .unwrap_or_else(|| get_env_var_or_exit("DATABASE_URL"));

        let pool = match PgPoolOptions::new()
            .max_connections(100)
            .connect(&db_url)
            .await
        {
            Ok(pool) => {
                println!("Successfully connected to the database.");
                pool
            }
            Err(err) => {
                println!("An error occurred connecting to the database: {err}");
                std::process::exit(1);
            }
        };

        let paths = args
            .option_target
            .into_iter()
            .chain(args.targets)
            .collect::<Vec<_>>();

        let mut prev_contents = String::new();
        println!("{}", "====================".truecolor(128, 128, 128));

        loop {
            let contents = fs::read_to_string(paths.first().unwrap()).unwrap_or_default();

            if prev_contents == contents {
                sleep(Duration::from_millis(200)).await;
                continue;
            }

            prev_contents = contents.clone();

            let mut queries = contents
                .split(';')
                .map(str::trim)
                .filter(|query| !query.is_empty())
                .enumerate()
                .peekable();

            while let Some((query_number, query)) = queries.next() {
                validate_query(&pool, format!("{query};"), query_number + 1).await;

                if !queries.peek().is_none() {
                    println!("{}", "--------------------".truecolor(128, 128, 128));
                }
            }

            sleep(Duration::from_millis(200)).await;
            println!("{}", "====================".truecolor(128, 128, 128));
        }
    } else if args.format {
        let paths = args
            .option_target
            .into_iter()
            .chain(args.targets)
            .collect::<Vec<_>>();

        if paths.is_empty() {
            let current_dir = env::current_dir()?;
            return poppy_sql::process_path(&current_dir);
        }

        for path in paths {
            poppy_sql::process_path(&path)?;
        }
    }

    Ok(())
}
