use std::{
    env::{self},
    fs::{self},
    io,
    path::Path,
};

fn main() {
    let current_dir = env::current_dir().unwrap();
    let dir = Path::new(&current_dir);

    traverse_dirs(dir).unwrap();
}

fn traverse_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                traverse_dirs(&path)?;
            } else {
                if !String::from(entry.file_name().to_str().unwrap_or("")).ends_with(".py") {
                    continue;
                }

                let filename = String::from(entry.file_name().to_str().unwrap_or(""));
                let contents = fs::read_to_string(&path).unwrap_or(String::from(""));

                println!("{filename}");
                println!("{contents}");
            }
        }
    }
    Ok(())
}
