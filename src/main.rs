use chrono::{DateTime, Utc};
use clap::Parser;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
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
use users::{Users, UsersCache};

#[derive(Debug, Display, Serialize)]
enum EntryType {
    File,
    Dir,
}

#[derive(Tabled)]
struct FileEntryShort {
    name: String,
}

#[derive(Debug, Tabled, Serialize)]
struct FileEntryLong {
    #[tabled(rename = "Permission")]
    permissions: String,
    #[tabled(rename = "Owner")]
    owner: String,
    #[tabled{rename="Name"}]
    name: String,
    #[tabled{rename="Type"}]
    e_type: EntryType,
    #[tabled{rename="Size B"}]
    len_bytes: u64,
    #[tabled(rename = "Modified")]
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

    #[arg(short, long, help = "Use a long listing format")]
    long: bool,

    #[arg(long, help = "List files in a tree-like format")]
    tree: bool,
}

fn main() {
    let cli = Cli::parse();
    let path = cli.path.as_ref().cloned().unwrap_or(PathBuf::from("."));

    if let Ok(does_exists) = fs::exists(&path) {
        if does_exists {
            if cli.tree {
                print_tree(&path, "", &cli);
            } else if cli.json {
                let files = get_long_files(&path, &cli);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&files).unwrap_or("cannot parse json".to_string())
                );
            } else if cli.long {
                print_long_table(&path, &cli);
            } else {
                print_short_table(&path, &cli)
            }
        } else {
            println!("{}", "Path does not exists".red());
        }
    } else {
        println!("{}", "error reading directory".red());
    }
}

fn print_short_table(path: &Path, cli: &Cli) {
    let files = get_short_files(path, cli);
    let table = Table::new(files);
    println!("{}", table);
}

fn get_short_files(path: &Path, cli: &Cli) -> Vec<FileEntryShort> {
    let mut data = Vec::new();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                let file_name_str = file.file_name().to_string_lossy().to_string();
                if !cli.all && file_name_str.starts_with(".") {
                    continue;
                }
                data.push(FileEntryShort {
                    name: file_name_str,
                });
            }
        }
    }
    data
}

fn print_long_table(path: &Path, cli: &Cli) {
    let get_files = get_long_files(path, cli);
    let mut table = Table::new(get_files);

    table.with(Style::rounded());
    table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);

    println!("{}", table);
}

fn get_long_files(path: &Path, cli: &Cli) -> Vec<FileEntryLong> {
    let mut data = Vec::new();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                let file_name_str = file.file_name().to_string_lossy().to_string();
                if !cli.all && file_name_str.starts_with(".") {
                    continue;
                }
                map_long_data(&mut data, file, cli);
            }
        }
    }
    data
}

// fn print_table(path: PathBuf, cli: &Cli) {
//     let get_files = get_files(&path, cli);
//     let mut table = Table::new(get_files);
//     table.with(Style::rounded());
//
//     table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
//     table.modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA);
//     table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
//     table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
//
//     println!("{}", table);
// }

// fn get_files(path: &Path, cli: &Cli) -> Vec<FileEntry> {
//     let mut data = Vec::default();
//     if let Ok(read_dir) = fs::read_dir(path) {
//         for entry in read_dir {
//             if let Ok(file) = entry {
//                 let file_name_str = file.file_name().to_string_lossy().to_string();
//
//                 if !cli.all && file_name_str.starts_with(".") {
//                     continue;
//                 }
//                 map_data(&mut data, file, cli);
//             }
//         }
//     }
//     data
// }

fn map_long_data(data: &mut Vec<FileEntryLong>, file: fs::DirEntry, _cli: &Cli) {
    let cache = UsersCache::new();
    if let Ok(meta) = fs::metadata(&file.path()) {
        let owner = cache
            .get_user_by_uid(meta.uid())
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or_else(|| meta.uid().to_string());

        data.push(FileEntryLong {
            permissions: format!("{:o}", meta.permissions().mode() & 0o777),
            owner,
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

fn print_tree(path: &Path, prefix: &str, cli: &Cli) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();

    entries.sort_by_key(|e| e.file_name());

    let mut peekable_entries = entries.into_iter().peekable();

    while let Some(entry) = peekable_entries.next() {
        let file_name_str = entry.file_name().to_string_lossy().to_string();
        if !cli.all && file_name_str.starts_with(".") {
            continue;
        }

        let is_last = peekable_entries.peek().is_none();
        let connector = if is_last { "└── " } else { " ├── " };

        println!("{}{}{}", prefix, connector, file_name_str.bright_blue());

        if entry.path().is_dir() {
            let new_prefix = if is_last { " " } else { "| " };
            print_tree(&entry.path(), &format!("{}{}", prefix, new_prefix), cli);
        }
    }
}
