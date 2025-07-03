use chrono::{DateTime, Utc};
use clap::Parser;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use strum::Display;
use tabled::{
    Table, Tabled,
    settings::{
        Color, Style,
        object::{Columns, Rows},
    },
};

#[derive(Debug, Display, Serialize)]
enum EntryType {
    File,
    Dir,
}

#[derive(Debug, Tabled, Serialize)]
struct FileEntry {
    #[tabled{rename="Name"}]
    name: String,
    #[tabled{rename="Type"}]
    e_type: EntryType,
    #[tabled{rename="Size B"}]
    len_bytes: u64,
    modified: String,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best ls command ever")]
struct Cli {
    path: Option<PathBuf>,
    #[arg(short, long)]
    json: bool,

    #[arg(short, long, help = "Show hidden files")]
    all: bool,
}

fn main() {
    let cli = Cli::parse();

    let path = cli.path.as_ref().cloned().unwrap_or(PathBuf::from("."));

    if let Ok(does_exists) = fs::exists(&path) {
        if does_exists {
            if cli.json {
                let get_files = get_files(&path, &cli);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&get_files)
                        .unwrap_or("cannot parse json".to_string())
                );
            } else {
                print_table(path, &cli);
            }
        } else {
            println!("{}", "Path does not exists".red());
        }
    } else {
        println!("{}", "error reading directory".red());
    }
}

fn print_table(path: PathBuf, cli: &Cli) {
    let get_files = get_files(&path, cli);
    let mut table = Table::new(get_files);
    table.with(Style::rounded());

    table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA);
    table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);

    println!("{}", table);
}

fn get_files(path: &Path, cli: &Cli) -> Vec<FileEntry> {
    let mut data = Vec::default();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                let file_name_str = file.file_name().to_string_lossy().to_string();

                if !cli.all && file_name_str.starts_with(".") {
                    continue;
                }
                map_data(&mut data, file, cli);
            }
        }
    }
    data
}

fn map_data(data: &mut Vec<FileEntry>, file: fs::DirEntry, cli: &Cli) {
    if let Ok(meta) = fs::metadata(&file.path()) {
        data.push(FileEntry {
            name: file
                .file_name()
                .into_string()
                .unwrap_or("unknown name".into()),
            e_type: if meta.is_dir() {
                EntryType::Dir
            } else {
                EntryType::File
            },
            len_bytes: meta.len(),
            modified: if let Ok(modi) = meta.modified() {
                let date: DateTime<Utc> = modi.into();
                format!("{}", date.format("%a %b %e %Y"))
            } else {
                String::default()
            },
        });
    }
}
