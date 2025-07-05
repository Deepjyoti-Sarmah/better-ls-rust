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
use tabled::settings::{Alignment, Width};
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
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    e_type: EntryType,
    #[tabled(rename = "Size B")]
    len_bytes: u64,
    #[tabled(rename = "Modified")]
    modified: String,
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
                print_tree(&path, &cli);
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
    let mut table = Table::new(files);

    table.with(Style::rounded());

    table.modify(Columns::new(..), Alignment::left());
    table.modify(Columns::new(2..3), Alignment::right());

    table.modify(Columns::new(0..1), Width::increase(15)); // Name
    table.modify(Columns::new(1..2), Width::increase(8)); // Type
    table.modify(Columns::new(2..3), Width::increase(10)); // Size
    table.modify(Columns::new(3..4), Width::increase(15)); // Modified

    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);

    table.modify(Columns::new(0..1), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::new(1..2), Color::FG_WHITE);
    table.modify(Columns::new(2..3), Color::FG_BRIGHT_MAGENTA);
    table.modify(Columns::new(3..4), Color::FG_BRIGHT_BLUE);

    println!("{}", table);
}

fn get_short_files(path: &Path, cli: &Cli) -> Vec<FileEntryShort> {
    let mut data = Vec::new();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                let file_name_str = file.file_name().to_string_lossy().to_string();
                if !cli.all && file_name_str.starts_with('.') {
                    continue;
                }
                map_short_data(&mut data, file, cli);
            }
        }
    }
    data
}

fn map_short_data(data: &mut Vec<FileEntryShort>, file: fs::DirEntry, _cli: &Cli) {
    if let Ok(meta) = fs::metadata(&file.path()) {
        let file_name = file
            .file_name()
            .into_string()
            .unwrap_or("unknown name".into());

        let display_name = file_name.clone();

        data.push(FileEntryShort {
            name: display_name,
            e_type: if meta.is_dir() {
                EntryType::Dir
            } else {
                EntryType::File
            },
            len_bytes: meta.len(),
            modified: if let Ok(modi) = meta.modified() {
                let data: DateTime<Utc> = modi.into();
                format!("{}", data.format("%a %b %e %Y"))
            } else {
                String::default()
            },
        });
    }
}

fn print_long_table(path: &Path, cli: &Cli) {
    let get_files = get_long_files(path, cli);
    let mut table = Table::new(get_files);

    table.with(Style::rounded());

    table.modify(Columns::new(..), Alignment::left());
    table.modify(Columns::new(4..5), Alignment::right());

    // Set minimum widths to prevent cramping
    table.modify(Columns::new(0..1), Width::increase(12)); // Permissions
    table.modify(Columns::new(1..2), Width::increase(12)); // Owner
    table.modify(Columns::new(2..3), Width::increase(20)); // Name
    table.modify(Columns::new(3..4), Width::increase(6)); // Type
    table.modify(Columns::new(4..5), Width::increase(10)); // Size
    table.modify(Columns::new(5..6), Width::increase(15));

    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);

    table.modify(Columns::new(0..1), Color::FG_BRIGHT_YELLOW); // Permissions
    table.modify(Columns::new(1..2), Color::FG_BRIGHT_WHITE); // Owner
    table.modify(Columns::new(2..3), Color::FG_BRIGHT_CYAN); // Name
    table.modify(Columns::new(3..4), Color::FG_WHITE); // Type
    table.modify(Columns::new(4..5), Color::FG_BRIGHT_MAGENTA); // Size
    table.modify(Columns::new(5..6), Color::FG_BRIGHT_BLUE); // Modified

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

fn map_long_data(data: &mut Vec<FileEntryLong>, file: fs::DirEntry, _cli: &Cli) {
    let cache = UsersCache::new();
    if let Ok(meta) = fs::metadata(&file.path()) {
        let owner = cache
            .get_user_by_uid(meta.uid())
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or_else(|| meta.uid().to_string());

        // Get the raw file name
        let file_name = file
            .file_name()
            .into_string()
            .unwrap_or("unknown name".into());

        let display_name = file_name.clone();

        data.push(FileEntryLong {
            permissions: format!("{:o}", meta.permissions().mode() & 0o777),
            owner,
            name: display_name, // Use the colored name here
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

// fn print_tree(path: &Path, prefix: &str, cli: &Cli) {
//     print_tree_with_depth(path, prefix, cli, 0, 3);
// }
//
// fn print_tree_with_depth(
//     path: &Path,
//     prefix: &str,
//     cli: &Cli,
//     current_depth: usize,
//     max_depth: usize,
// ) {
//     if current_depth >= max_depth {
//         return;
//     }
//
//     let Ok(entries) = fs::read_dir(path) else {
//         return;
//     };
//     let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
//
//     entries.sort_by_key(|e| e.file_name());
//
//     let mut peekable_entries = entries.into_iter().peekable();
//
//     while let Some(entry) = peekable_entries.next() {
//         let file_name_str = entry.file_name().to_string_lossy().to_string();
//         if !cli.all && file_name_str.starts_with(".") {
//             continue;
//         }
//
//         let is_last = peekable_entries.peek().is_none();
//         let connector = if is_last { "└── " } else { " ├── " };
//
//         println!("{}{}{}", prefix, connector, file_name_str.bright_blue());
//
//         if entry.path().is_dir() {
//             let new_prefix = if is_last { " " } else { "| " };
//             print_tree_with_depth(
//                 &entry.path(),
//                 &format!("{}{}", prefix, new_prefix),
//                 cli,
//                 current_depth + 1,
//                 max_depth,
//             );
//         }
//     }
// }

fn print_tree(path: &Path, cli: &Cli) {
    // Print the root directory name
    let root_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    println!("{}", root_name.bright_blue().bold());

    print_tree_recursive(path, "", cli, 0, 3, true);
}

fn print_tree_recursive(
    path: &Path,
    prefix: &str,
    cli: &Cli,
    current_depth: usize,
    max_depth: usize,
    is_root: bool,
) {
    if current_depth >= max_depth {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();

    // Sort entries: directories first, then files, both alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    // Filter out hidden files if needed
    let mut visible_entries: Vec<_> = entries
        .into_iter()
        .filter(|entry| {
            let file_name_str = entry.file_name().to_string_lossy().to_string();
            cli.all || !file_name_str.starts_with(".")
        })
        .collect();

    for (index, entry) in visible_entries.iter().enumerate() {
        let is_last = index == visible_entries.len() - 1;
        let file_name_str = entry.file_name().to_string_lossy().to_string();
        let is_directory = entry.path().is_dir();

        // Choose the appropriate tree characters
        let (connector, next_prefix) = if is_last {
            ("└── ", format!("{}    ", prefix))
        } else {
            ("├── ", format!("{}│   ", prefix))
        };

        // Color the file name based on type
        let colored_name = if is_directory {
            file_name_str.bright_blue().bold().to_string()
        } else {
            // Check file extension for different colors
            let extension = Path::new(&file_name_str)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            match extension {
                "rs" | "py" | "js" | "ts" | "go" | "cpp" | "c" | "java" => {
                    file_name_str.bright_green().to_string()
                }
                "txt" | "md" | "readme" => file_name_str.bright_yellow().to_string(),
                "json" | "yaml" | "yml" | "toml" | "xml" => file_name_str.bright_cyan().to_string(),
                "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => {
                    file_name_str.bright_magenta().to_string()
                }
                _ => file_name_str.white().to_string(),
            }
        };

        // Print the current entry
        println!("{}{}{}", prefix, connector, colored_name);

        // Recursively print subdirectories
        if is_directory {
            print_tree_recursive(
                &entry.path(),
                &next_prefix,
                cli,
                current_depth + 1,
                max_depth,
                false,
            );
        }
    }
}
