use clap::{Parser, Subcommand};
use std::path::Path;
use std::time::Duration;

mod hydrate;
mod peers;
use crate::peers::find_peers;
use sqlx::postgres::{PgPool, PgPoolOptions};
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

        #[arg(
            long,
            default_value = "false",
            help = "Keep the git clone after hydrating"
        )]
        keep_clone: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(std::env::var("DATABASE_URL").unwrap().as_str())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    match cli.sub {
        Sub::Serve { port } => println!("Serving on port {}", port),
        Sub::Hydrate {
            git_remote,
            git_ref,
            path,
            keep_clone,
        } => hydrate_chain_registry(pool, git_remote, git_ref, path, keep_clone).await,
    }
}

async fn hydrate_chain_registry(
    pool: PgPool,
    remote: String,
    git_ref: String,
    path: Option<String>,
    keep_clone: bool,
) {
    let clone_dir =
        path.unwrap_or_else(|| TempDir::new().unwrap().path().to_str().unwrap().to_string());
    println!("Cloning {} {} into {}...", remote, git_ref, clone_dir);
    let repo = hydrate::shallow_clone(remote, git_ref, &clone_dir.clone().into())
        .expect("shallow clone failed");

    let mut conn = pool.acquire().await.unwrap();

    // chain ids mutable array of i64
    let mut chain_ids: Vec<i64> = Vec::new();

    println!("Inserting chains...");

    // Insert mainnet chains
    for chain in repo.mainnets {
        match hydrate::insert_chain(
            &mut conn,
            chain.to_path_buf(),
            "mainnet".to_string(),
            &repo.commit,
        )
        .await
        {
            Ok(id) => {
                chain_ids.push(id);
            }
            Err(err) => println!("Failed to save mainnet chain {:?}: {:?}", chain, err),
        }
    }

    // Insert testnet chains
    for chain in repo.testnets {
        match hydrate::insert_chain(
            &mut conn,
            chain.to_path_buf(),
            "testnet".to_string(),
            &repo.commit,
        )
        .await
        {
            Ok(id) => {
                chain_ids.push(id);
            }
            Err(err) => println!("Failed to save testnet chain {:?}: {:?}", chain, err),
        }
    }

    // Insert peers
    let pool = &pool;
    let mut handles = vec![];

    println!("Inserting peers...");
    for chain_id in chain_ids {
        // insert_peer(&pool, chain_id).await;
        handles.push(tokio::spawn(async move {
            insert_peer(pool, chain_id).await;
        }));
    }

    if keep_clone {
        return;
    }

    let path = Path::new(clone_dir.as_str());
    match std::fs::remove_dir_all(path) {
        Ok(_) => {
            println!("Removed clone dir {}", clone_dir)
        }
        Err(err) => println!("Failed to remove clone dir: {:?}", err),
    }

    for handle in handles {
        handle.await.unwrap(); // TODO fix
    }
}

async fn insert_peer(pool: &PgPool, chain_id: i64) {
    let mut conn = match pool.acquire().await {
        Ok(conn) => conn,
        Err(err) => {
            println!("Failed to acquire connection: {:?}", err);
            return;
        }
    };
    for peer_type in [peers::PeerType::Seed, peers::PeerType::Persistent].into_iter() {
        let peers = match find_peers(&mut conn, chain_id, peer_type.clone()).await {
            Ok(peers) => peers,
            Err(err) => {
                println!("Failed to find peers {:?}: {:?}", peer_type, err);
                continue;
            }
        };

        let check_liveness = |addr: &str| -> anyhow::Result<()> {
            println!("Checking liveness of {}", addr);
            peers::tcp_check_liveness(addr, Duration::from_secs(5))
        };

        for peer in peers {
            match peers::insert_peer(
                &mut conn,
                chain_id,
                peer_type.clone(),
                peer.clone(),
                check_liveness,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => println!("Failed to insert peer {:?}: {:?}", peer, err),
            }
        }
    }
}
