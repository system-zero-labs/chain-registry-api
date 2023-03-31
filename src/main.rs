use clap::Parser;
use std::time::Duration;
use tempfile::TempDir;
use tokio_cron_scheduler::{Job, JobScheduler};
mod hydrate;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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
        help = "Path to dir for git clone. If empty, defaults to tmp directory",
        required = false
    )]
    clone_path: Option<String>,

    #[arg(
        short,
        long,
        default_value = "* */10 * * * *",
        help = "How often to poll the chain registry repo for new updates",
        env = "CHAIN_REGISTRY_API_CRON"
    )]
    cron: String,

    #[arg(
        short,
        long,
        default_value = "8675",
        help = "Port to bind",
        env = "PORT"
    )]
    port: u16,
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    let clone_dir = cli
        .clone_path
        .unwrap_or_else(|| TempDir::new().unwrap().path().to_str().unwrap().to_string());

    let sched = JobScheduler::new().await.unwrap();

    // Clone immediately when command is run.
    sched
        .add(
            Job::new_one_shot(Duration::from_secs(0), move |_uuid, _l| {
                hydrate_chain_registry(
                    cli.git_remote.clone(),
                    cli.git_ref.clone(),
                    clone_dir.clone(),
                )
            })
            .unwrap(),
        )
        .await
        .unwrap();

    sched.start().await.unwrap();

    // Wait a while so that the jobs actually run
    tokio::time::sleep(Duration::from_secs(100)).await;
}

fn hydrate_chain_registry(remote: String, git_ref: String, clone_dir: String) {
    println!("Cloning {} {} into {}...", remote, git_ref, clone_dir);
    match hydrate::shallow_clone(remote, git_ref, clone_dir.into()) {
        Ok(_) => println!("Clone successful"), // TODO: use the ChainDirs
        Err(e) => println!("Error: {}", e),
    }
}
