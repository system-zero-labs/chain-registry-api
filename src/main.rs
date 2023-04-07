use clap::{Parser, Subcommand};
use sqlx::pool::PoolConnection;
use std::path::Path;
use std::time::Duration;

mod hydrate;
mod liveness;
mod peers;

use crate::peers::find_peers;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tempfile::TempDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    sub: Sub,

    #[arg(
        long,
        help = "Max number of postgres connections",
        default_value = "10",
        global = true
    )]
    pg_conns: u32,

    #[arg(
        long,
        help = "Postgres connection timeout in seconds",
        default_value = "600",
        global = true
    )]
    pg_timeout_sec: u64,
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

    #[command(about = "Check liveness of peers and rpc/api endpoints")]
    Liveness,
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
            keep_clone,
        } => hydrate_chain_registry(git_remote, git_ref, path, keep_clone).await,
        Sub::Liveness => {
            check_liveness(cli.pg_conns, Duration::from_secs(cli.pg_timeout_sec)).await;
        }
    }
}

async fn connect_pool(max_conns: u32, timeout: Duration) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(max_conns)
        .acquire_timeout(timeout)
        .connect(std::env::var("DATABASE_URL").unwrap().as_str())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

async fn hydrate_chain_registry(
    remote: String,
    git_ref: String,
    path: Option<String>,
    keep_clone: bool,
) {
    let pool = connect_pool(2, Duration::from_secs(30)).await;

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

    println!("Inserting peers...");
    for chain_id in chain_ids {
        insert_peer(&mut conn, chain_id).await;
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
}

async fn insert_peer(conn: &mut PoolConnection<sqlx::Postgres>, chain_id: i64) {
    for peer_type in [peers::PeerType::Seed, peers::PeerType::Persistent].into_iter() {
        let peers = match find_peers(conn, chain_id, peer_type.clone()).await {
            Ok(peers) => peers,
            Err(err) => {
                println!("Failed to find peers {:?}: {:?}", peer_type, err);
                continue;
            }
        };

        for peer in peers {
            match peers::insert_peer(conn, chain_id, peer_type.clone(), peer.clone()).await {
                Ok(_) => {}
                Err(err) => println!("Failed to insert peer {:?}: {:?}", peer, err),
            }
        }
    }
}

async fn check_liveness(max_conns: u32, timeout: Duration) {
    let pool = connect_pool(max_conns, timeout).await;

    let mut conn = pool
        .acquire()
        .await
        .expect("Failed to acquire connection from pool");

    let peers = peers::recent_peers(&mut conn)
        .await
        .expect("Failed to get recent peers");

    let pool = std::sync::Arc::new(pool);
    let mut handles = vec![];
    for peer in peers {
        let clone = std::sync::Arc::clone(&pool);
        handles.push(tokio::spawn(async move {
            let mut conn = match clone.acquire().await {
                Ok(conn) => conn,
                Err(err) => {
                    println!("Failed to acquire connection from pool: {:?}", err);
                    return;
                }
            };

            let check_liveness = |addr: &str| -> anyhow::Result<()> {
                println!("Checking peer liveness for {}", addr);
                liveness::tcp_check_liveness(addr, Duration::from_secs(5))
            };

            match peers::update_liveness(&mut conn, &peer, check_liveness).await {
                Ok(_) => {}
                Err(err) => println!("Failed to update liveness for {:?}: {:?}", peer, err),
            }
        }));
    }

    for handle in handles {
        match handle.await {
            Ok(_) => {}
            Err(err) => println!("Task failed: {:?}", err),
        }
    }
}
