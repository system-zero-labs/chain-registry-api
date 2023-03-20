use clap::{Parser, Subcommand};
mod hydrate;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    sub: Sub,
}

#[derive(Subcommand, Debug)]
enum Sub {
    #[command(about = "Run the API server")]
    Serve {
        #[arg(
            short,
            long,
            default_value = "8675",
            help = "Port to bind",
            env = "PORT"
        )]
        port: u16,
    },

    #[command(about = "Download data from Chain Registry and store in database")]
    Hydrate {
        #[arg(
            long,
            default_value = "https://github.com/cosmos/chain-registry",
            help = "Chain Registry git URL"
        )]
        git_remote: String,

        #[arg(long, default_value = "master", help = "Git branch or tag")]
        git_ref: String,

        #[arg(
            long,
            default_value = "chain-registry",
            help = "Path to dir for git clone"
        )]
        path: String,
    },
}

fn main() {
    let cli = Args::parse();

    match cli.sub {
        Sub::Serve { port } => println!("Serving on port {}", port),
        Sub::Hydrate {
            git_remote,
            git_ref,
            path,
        } => {
            println!("Cloning into {}...", path);
            hydrate::shallow_clone(git_remote, git_ref, path).unwrap();
        }
    }
}
