use sqlx::pool::PoolConnection;
use std::fs;
use std::path::PathBuf;

pub struct ChainDirs {
    pub mainnets: Vec<PathBuf>,
    pub testnets: Vec<PathBuf>,
}

pub fn shallow_clone(
    remote: String,
    git_ref: String,
    clone_dir: &PathBuf,
) -> anyhow::Result<ChainDirs> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(git_ref)
        .arg(remote)
        .arg(clone_dir.to_str().unwrap());
    let output = cmd.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git clone failed with status: {:?} stderr: {:?} stdout: {:?}",
            output.status,
            std::str::from_utf8(output.stderr.as_ref()).unwrap_or("cannot read stderr"),
            std::str::from_utf8(output.stdout.as_ref()).unwrap_or("cannot read stdout")
        );
    }

    let mainnets = collect_chains(clone_dir.clone())?;
    let testnets = collect_chains(clone_dir.join("testnets"))?;
    Ok(ChainDirs { mainnets, testnets })
}

fn collect_chains(dir: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let found = fs::read_dir(dir)?
        .filter_map(|f| {
            let f = f.unwrap().path();
            if !f.is_dir() {
                return None;
            }
            return Some(f);
        })
        .filter(|f| {
            let fname = f.file_name().unwrap().to_str().unwrap();
            !(fname.starts_with("_") || fname.starts_with(".") || fname.starts_with("testnets"))
        })
        .collect();
    Ok(found)
}

pub async fn insert_chain(
    conn: &mut PoolConnection<sqlx::Postgres>,
    path: PathBuf,
    network: String,
) -> anyhow::Result<i64> {
    let chain_name = path.file_name().unwrap().to_str().unwrap();
    let chain_json = match fs::read_to_string(path.join("chain.json")) {
        Ok(c) => c,
        Err(err) => anyhow::bail!(
            "failed to read chain.json for chain {} {}: {:?}",
            chain_name,
            network,
            err,
        ),
    };
    let chain_json: serde_json::Value = serde_json::from_str(&chain_json)?;

    let assets_json = fs::read_to_string(path.join("assetlist.json")).unwrap_or("{}".to_string());
    let assets_json: serde_json::Value = serde_json::from_str(&assets_json)?;

    match sqlx::query!(
        r#"
        INSERT INTO chain (name, network, chain_data, asset_data)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        chain_name,
        network,
        chain_json,
        assets_json,
    )
    .fetch_one(conn)
    .await
    {
        Ok(row) => Ok(row.id),
        Err(err) => anyhow::bail!(
            "failed to insert chain {} {}: {:?}",
            chain_name,
            network,
            err,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{insert_chain, shallow_clone};
    use sqlx::PgPool;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_shallow_clone() {
        let temp_dir = TempDir::new().unwrap();

        let chains = shallow_clone(
            "https://github.com/cosmos/chain-registry".to_string(),
            "master".to_string(),
            &temp_dir.path().to_path_buf(),
        )
        .unwrap();

        assert!(temp_dir.path().join("cosmoshub/chain.json").exists());

        assert!(chains.mainnets.len() > 1);
        assert!(chains.mainnets.iter().all(|p| p.exists() && p.is_dir()));

        let mainnets: Vec<String> = chains
            .mainnets
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(
            mainnets.contains(&"cosmoshub".to_string()),
            "{:?}",
            mainnets
        );
        assert!(mainnets.contains(&"osmosis".to_string()), "{:?}", mainnets);
        assert!(!mainnets.contains(&".".to_string()), "{:?}", mainnets);
        assert!(!mainnets.contains(&".git".to_string()), "{:?}", mainnets);
        assert!(!mainnets.contains(&".github".to_string()), "{:?}", mainnets);
        assert!(
            !mainnets.contains(&"testnets".to_string()),
            "{:?}",
            mainnets
        );

        assert!(chains.testnets.len() > 1);
        assert!(chains.testnets.iter().all(|p| p.exists() && p.is_dir()));

        let testnets: Vec<String> = chains
            .testnets
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(testnets.len() > 1);
        assert!(
            testnets.contains(&"cosmoshubtestnet".to_string()),
            "{:?}",
            testnets
        );
        assert!(!testnets.contains(&".".to_string()));
    }

    #[sqlx::test]
    async fn test_insert_chain(pool: PgPool) -> sqlx::Result<()> {
        let test_path = TempDir::new().unwrap().into_path().join("cosmos");
        fs::create_dir(test_path.clone()).unwrap();
        let mut file = File::create(test_path.clone().join("chain.json"))?;
        file.write_all(b"stub chain data")?;
        let mut file = File::create(test_path.clone().join("assetlist.json"))?;
        file.write_all(b"stub asset data")?;

        let mut conn = pool.acquire().await?;

        let id = insert_chain(&mut conn, test_path.clone(), "testnet".to_string())
            .await
            .unwrap();

        assert_ne!(id, 0);

        let chain = sqlx::query!("SELECT * FROM chain")
            .fetch_one(&mut conn)
            .await?;
        assert_eq!(chain.name, "cosmos");
        assert_eq!(chain.network, "testnet");
        assert_eq!(chain.chain_data, "stub chain data");
        assert_eq!(chain.asset_data, "stub asset data");

        Ok(())
    }
}
