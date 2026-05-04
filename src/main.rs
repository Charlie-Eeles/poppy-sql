use std::{env, io, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long = "file", value_name = "FILE")]
    files_from_option: Vec<PathBuf>,

    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let paths = args
        .files_from_option
        .into_iter()
        .chain(args.files)
        .collect::<Vec<_>>();

    if paths.is_empty() {
        let current_dir = env::current_dir()?;
        return poppy_sql::process_path(&current_dir);
    }

    for path in paths {
        poppy_sql::process_path(&path)?;
    }

    Ok(())
}
