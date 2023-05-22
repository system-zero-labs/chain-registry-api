use sqlx::{types::JsonValue, PgExecutor};
use std::fs;
use std::path::PathBuf;

pub async fn insert_chain(
    executor: impl PgExecutor<'_>,
    path: PathBuf,
    network: String,
    commit: &String,
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

    // DO NOTHING causes a RowNotFound error. We want the updated_at triggers to fire as well, so
    // we update.
    match sqlx::query!(
        r#"
        INSERT INTO chain (name, network, chain_data, asset_data, commit)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (name, network, commit) DO UPDATE SET commit = $5
        RETURNING id
        "#,
        chain_name,
        network,
        chain_json,
        assets_json,
        commit,
    )
    .fetch_one(executor)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn as_str(&self) -> &str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
        }
    }

    pub fn from_str(s: &str) -> anyhow::Result<Network> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            _ => anyhow::bail!(
                "invalid network: {}, must be {} or {}",
                s,
                Network::Mainnet.as_str(),
                Network::Testnet.as_str()
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chain {
    pub id: i64,
    pub commit: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub chain_data: JsonValue,
    pub asset_data: JsonValue,
}

impl Chain {
    pub async fn from_name(
        executor: impl PgExecutor<'_>,
        name: &str,
        network: &Network,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
        Chain,
        r#"
        SELECT id, commit, created_at, chain_data, asset_data FROM chain WHERE name = $1 AND network = $2 ORDER BY created_at DESC LIMIT 1
        "#,
        name,
        network.as_str(),
    ).fetch_one(executor).await
    }
}

pub async fn truncate_old_chains(executor: impl PgExecutor<'_>, keep: i64) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        WITH recent AS (SELECT commit, max(created_at) AS created_at
                FROM chain
                GROUP BY commit
                ORDER BY created_at DESC
                LIMIT $1)
        DELETE FROM chain WHERE commit NOT IN (SELECT commit from recent)
        "#,
        keep,
    )
    .execute(executor)
    .await?;

    Ok(())
}

#[derive(Debug)]
pub struct ChainList {
    pub commit: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub names: Vec<String>,
}

pub async fn recent_chains(
    executor: impl PgExecutor<'_>,
    network: &str,
) -> sqlx::Result<ChainList> {
    let row = sqlx::query!(
        r#"
        SELECT commit, 
        array_agg(name order by name) as names, 
        MAX(created_at) as created_at 
        FROM chain WHERE network = $1 GROUP BY commit ORDER BY MAX(created_at) DESC LIMIT 1;
        "#,
        network,
    )
    .fetch_one(executor)
    .await?;

    Ok(ChainList {
        commit: row.commit,
        created_at: row.created_at.unwrap_or(chrono::Utc::now()),
        names: row.names.unwrap_or(vec![]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPool;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[sqlx::test]
    async fn test_insert_chain(pool: PgPool) -> sqlx::Result<()> {
        let test_path = TempDir::new().unwrap().into_path().join("cosmos");
        fs::create_dir(test_path.clone()).unwrap();
        let mut file = File::create(test_path.clone().join("chain.json"))?;
        file.write_all(r#"{"chain_id":"cosmoshub-4"}"#.as_bytes())?;
        let mut file = File::create(test_path.clone().join("assetlist.json"))?;
        let stub_asset_data = r#"{"stub":"data"}"#;
        file.write_all(stub_asset_data.as_bytes())?;

        let mut conn = pool.acquire().await?;

        let id = insert_chain(
            &mut conn,
            test_path.clone(),
            "testnet".to_string(),
            &"stub commit".to_string(),
        )
        .await
        .unwrap();

        assert_ne!(id, 0);

        let chain = sqlx::query!("SELECT * FROM chain")
            .fetch_one(&mut conn)
            .await?;
        assert_eq!(chain.name, "cosmos");
        assert_eq!(chain.network, "testnet");
        assert_eq!(chain.asset_data.to_string(), stub_asset_data);
        assert_eq!(chain.commit, "stub commit");

        // Ensure we saving JSON objects
        let chain = sqlx::query!(
            "SELECT chain_data->>'chain_id' as chain_id FROM chain WHERE id = $1",
            id
        )
        .fetch_one(&mut conn)
        .await?;

        assert_eq!(chain.chain_id.unwrap(), "cosmoshub-4");

        // Ensure we don't insert duplicate chains
        insert_chain(
            &mut conn,
            test_path.clone(),
            "testnet".to_string(),
            &"stub commit".to_string(),
        )
        .await
        .unwrap();

        let count = sqlx::query!("SELECT count(*) FROM chain")
            .fetch_one(&mut conn)
            .await?
            .count
            .unwrap();
        assert_eq!(count, 1);

        Ok(())
    }

    #[sqlx::test(fixtures("chains"))]
    async fn test_from_name(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let chain = Chain::from_name(&mut conn, "cosmoshub", &Network::Mainnet).await?;

        assert_eq!(chain.commit, "stubcommit");

        assert!(chain.chain_data.is_object());
        assert_eq!(
            chain.chain_data.get("chain_id").unwrap().as_str(),
            Some("cosmoshub-4")
        );

        assert!(chain.asset_data.is_object());
        assert_eq!(
            chain.asset_data.get("chain_name").unwrap().as_str(),
            Some("cosmoshub")
        );

        Ok(())
    }

    #[sqlx::test(fixtures("chains"))]
    async fn test_recent_chains(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let list = recent_chains(&mut conn, "mainnet").await?;

        assert_eq!(list.commit, "stubcommit");
        assert_eq!(list.names, vec!["cosmoshub"]);

        Ok(())
    }

    #[sqlx::test(fixtures("truncate_chains"))]
    async fn test_truncate_old_chains(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        truncate_old_chains(&mut conn, 2).await?;

        let commits = sqlx::query!("SELECT commit FROM chain")
            .fetch_all(&mut conn)
            .await?;

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].commit, "commit3");
        assert_eq!(commits[1].commit, "commit4");

        Ok(())
    }
}
