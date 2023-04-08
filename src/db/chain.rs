use sqlx::pool::PoolConnection;
use std::fs;
use std::path::PathBuf;

pub async fn insert_chain(
    conn: &mut PoolConnection<sqlx::Postgres>,
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
}
