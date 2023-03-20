use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    sub: Sub,
}

#[derive(Subcommand, Debug)]
enum Sub {
    #[command(about = "Run the API server")]
    Serve {
        #[arg(short, long, default_value = "8675", help = "Port to bind", env = "PORT")]
        port: u16,
    },
    #[command(about = "Download data from Chain Registry and store in database")]
    Hydrate {},
}

fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);

    match cli.sub {
        Sub::Serve { port } => println!("Serving on port {}", port),
        Sub::Hydrate {} => println!("Hydrating: TODO"),
    }
}
