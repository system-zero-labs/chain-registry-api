use clap::{Parser, Subcommand};
mod hydrate;
use tempfile::TempDir;

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

        #[arg(long, help = "Path to dir for git clone", required = false)]
        path: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();

    match cli.sub {
        Sub::Serve { port } => println!("Serving on port {}", port),
        Sub::Hydrate {
            git_remote,
            git_ref,
            path,
        } => hydrate_chain_registry(git_remote, git_ref, path).await,
    }
}

async fn hydrate_chain_registry(remote: String, git_ref: String, path: Option<String>) {
    let clone_dir =
        path.unwrap_or_else(|| TempDir::new().unwrap().path().to_str().unwrap().to_string());
    println!("Cloning {} {} into {}...", remote, git_ref, clone_dir);
    let chains =
        hydrate::shallow_clone(remote, git_ref, clone_dir.into()).expect("shallow clone failed");
    for chain in chains.mainnets {
        match hydrate::save_chain(todo, chain.to_path_buf(), "mainnet".to_string()).await {
            Ok(_) => {}
            Err(err) => println!("Failed to save mainnet chain {:?}: {:?}", chain, err),
        }
    }
}
