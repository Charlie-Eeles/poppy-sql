use std::{env, io};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    file: Option<std::path::PathBuf>,
}

fn main() -> io::Result<()> {
    let arg = Args::parse();

    if let Some(path) = arg.file {
        return poppy_sql::process_path(&path);
    }

    let current_dir = env::current_dir()?;
    poppy_sql::traverse_dirs(&current_dir)
}
