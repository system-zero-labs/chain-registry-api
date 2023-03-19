use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
}

fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli)
}
