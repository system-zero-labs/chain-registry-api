use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawPeer {
    node_id: String,
    address: String,
    provider: Option<String>,
}

pub async fn insert_persistent_peer(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    chainID: i64,
) -> anyhow::Result<()> {
    anyhow::bail!("Not implemented");
}

#[cfg(test)]
mod tests {
    use crate::peers::insert_persistent_peer;
    use sqlx::PgPool;

    #[sqlx::test(fixtures("chains"))]
    async fn test_save_chain(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        insert_persistent_peer(&mut conn, 1).await.unwrap();

        Ok(())
    }
}
