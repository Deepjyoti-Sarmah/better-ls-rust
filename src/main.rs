use clap::Parser;
use owo_colors::OwoColorize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use strum::Display;
use tabled::{
    Table, Tabled,
    settings::{
        Color, Style,
        object::{Column, Columns, Row, Rows},
        span::ColumnSpan,
    },
};

#[derive(Debug, Display)]
enum EntryType {
    File,
    Dir,
}

#[derive(Debug, Tabled)]
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
}

fn main() {
    let cli = Cli::parse();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(does_exists) = fs::exists(&path) {
        if does_exists {
            let get_files = get_files(&path);
            let mut table = Table::new(get_files);
            table.with(Style::rounded());
            table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
            table.modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA);
            table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
            table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
            println!("{}", table);
        } else {
            println!("{}", "Path does not exists".red());
        }
    } else {
        println!("{}", "error reading directory".red());
    }
}

fn get_files(path: &Path) -> Vec<FileEntry> {
    let mut data = Vec::default();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                map_data(&mut data, file);
            }
        }
    }

    data
}

fn map_data(data: &mut Vec<FileEntry>, file: fs::DirEntry) {
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
            modified: "".to_string(),
        });
    }
}
