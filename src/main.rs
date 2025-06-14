use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best ls command ever")]
struct Cli {
    path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    println!("Hello, world!");
}
