use clap::Parser;
use owo_colors::OwoColorize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use strum::Display;

#[derive(Debug, Display)]
enum EntryType {
    File,
    Dir,
}

#[derive(Debug)]
struct FileEntry {
    name: String,
    len_bytes: u64,
    modified: String,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best ls command ever")]
struct Cli {
    path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(does_exists) = fs::exists(&path) {
        if does_exists {
            for file in get_files(&path) {
                println!("{}", file)
            }
        } else {
            println!("{}", "Path does not exists".red());
        }
    } else {
        println!("{}", "error reading directory".red());
    }

    // println!("{}", path.display());
}

fn get_files(path: &Path) -> Vec<String> {
    let mut data = Vec::default();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                data.push(file.file_name().into_string().unwrap_or("unknown".into()));
            }
        }
    }

    data
}
