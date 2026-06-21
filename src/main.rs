use std::{env, fs, io, path::PathBuf};

use clap::Parser;
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

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.watch {
        println!("watch mode enabled");
        if args.db_url.is_none() {
            println!("db_url argument required for watch command.");
            std::process::exit(1);
        }

        let pool = match PgPoolOptions::new()
            .max_connections(100)
            .connect(&args.db_url.unwrap_or(String::from("")))
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
        loop {
            let contents = fs::read_to_string(paths.first().unwrap()).unwrap_or_default();
            if prev_contents == contents {
                sleep(Duration::from_millis(200)).await;
                continue;
            };
            prev_contents = String::from(&contents);
            validate_query(&pool, contents).await;
            sleep(Duration::from_millis(200)).await;
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
