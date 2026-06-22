use std::{env, fs, io, path::PathBuf, time::Instant};

use clap::Parser;
use colored::Colorize;
use dotenv::dotenv;
use notify::{Event, RecursiveMode, Result, Watcher};
use poppy_sql::watch::handlers::validate_query;
use sqlx::postgres::PgPoolOptions;
use std::{path::Path, sync::mpsc};
use tokio::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'f', long)]
    format: bool,

    #[arg(short = 'w', long)]
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

    let paths = args
        .option_target
        .into_iter()
        .chain(args.targets)
        .collect::<Vec<_>>();

    //TODO: This logic shouldn't be in main - this should be properly abstracted
    if args.watch {
        if paths.is_empty() {
            println!("No target paths provided.");
            std::process::exit(1);
        }

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

        println!("{}", "====================".dimmed());
        println!("Watching for file changes.");
        println!("{}", "====================".dimmed());

        let (tx, rx) = mpsc::channel::<Result<Event>>();

        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(watcher) => watcher,
            Err(err) => {
                println!("An error occurred creating watcher: {err}");
                std::process::exit(1);
            }
        };

        let debounce_duration = Duration::from_millis(200);
        let mut last_event_time = Instant::now() - debounce_duration;

        match watcher.watch(Path::new("."), RecursiveMode::NonRecursive) {
            Ok(_) => {}
            Err(err) => {
                println!("An error occurred starting watcher: {err}");
                std::process::exit(1);
            }
        };

        for res in rx {
            match res {
                Ok(event) => {
                    if !matches!(
                        event.kind,
                        notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
                    ) {
                        continue;
                    }

                    let now = Instant::now();
                    if now.duration_since(last_event_time) < debounce_duration {
                        continue;
                    }
                    last_event_time = now;

                    for (_, path) in paths.iter().enumerate() {
                        let mut contents = fs::read_to_string(path).unwrap_or_default();

                        if args.format {
                            poppy_sql::process_path(path)?;
                            contents = fs::read_to_string(path).unwrap_or_default();
                        }

                        println!("{}", "====================".dimmed());

                        let mut queries = contents
                            .split(';')
                            .map(str::trim)
                            .filter(|query| !query.is_empty())
                            .enumerate()
                            .peekable();

                        while let Some((query_number, query)) = queries.next() {
                            validate_query(&pool, format!("{query};"), query_number + 1).await;

                            if queries.peek().is_some() {
                                println!("{}", "--------------------".dimmed());
                            }
                        }

                        println!("{}", "====================".dimmed());
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    } else if args.format {
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
