use sqlx::PgExecutor;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "endpoint_kind")]
#[sqlx(rename_all = "lowercase")]
pub enum EndpointKind {
    Peer,
    Seed,
    Rpc,
    Rest,
    Grpc,
}

impl EndpointKind {
    fn as_field(&self) -> &str {
        match self {
            EndpointKind::Peer => "persistent_peers",
            EndpointKind::Seed => "seeds",
            EndpointKind::Rpc => "rpc",
            EndpointKind::Rest => "rest",
            EndpointKind::Grpc => "grpc",
        }
    }
}

pub async fn insert_persistent_peers(
    executor: impl PgExecutor<'_>,
    chain_id: i64,
) -> sqlx::Result<Vec<i64>> {
    insert_peers(executor, chain_id, EndpointKind::Peer).await
}

pub async fn insert_seeds(executor: impl PgExecutor<'_>, chain_id: i64) -> sqlx::Result<Vec<i64>> {
    insert_peers(executor, chain_id, EndpointKind::Seed).await
}

async fn insert_peers(
    executor: impl PgExecutor<'_>,
    chain_id: i64,
    kind: EndpointKind,
) -> sqlx::Result<Vec<i64>> {
    let ids = sqlx::query!(
        r#"
        WITH cte AS (
        SELECT
            chain_data ->> 'chain_id' as chain_id,
            jsonb_array_elements(chain_data -> 'peers' -> $1) ->> 'id' as node_id,
            jsonb_array_elements(chain_data -> 'peers' -> $1) ->> 'address' as address,
            jsonb_array_elements(chain_data -> 'peers' -> $1) ->> 'provider' as provider
            FROM chain
            WHERE id = $2
        )
        INSERT INTO endpoint (chain_id, address, provider, kind)
        SELECT
            chain_id,
            CONCAT(node_id, '@', address),
            COALESCE(provider, 'Unknown'),
            $3
        FROM cte
        WHERE chain_id IS NOT NULL AND node_id IS NOT NULL AND address IS NOT NULL
        ON CONFLICT (address, kind) DO UPDATE SET provider = EXCLUDED.provider
        RETURNING id
    "#,
        kind.as_field(),
        chain_id,
        kind as _,
    )
    .fetch_all(executor)
    .await?;

    Ok(ids.into_iter().map(|row| row.id).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(fixtures("chains"))]
    async fn test_insert_persistent_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let ids = insert_persistent_peers(&mut conn, 1).await?;
        assert_eq!(ids.len(), 3);

        // Tests ON CONFLICT
        let next_ids = insert_persistent_peers(&mut conn, 1).await?;
        assert_eq!(ids, next_ids);

        Ok(())
    }

    #[sqlx::test(fixtures("chains"))]
    async fn test_insert_seeds(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let ids = insert_seeds(&mut conn, 1).await?;
        assert_eq!(ids.len(), 7);

        // Tests ON CONFLICT
        let next_ids = insert_seeds(&mut conn, 1).await?;
        assert_eq!(ids, next_ids);

        Ok(())
    }
}
