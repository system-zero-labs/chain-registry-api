use clap::{Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "8765", help = "Port to bind", env = "PORT")]
    port: u16,
}


fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);
}
