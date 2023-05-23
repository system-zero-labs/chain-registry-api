use super::chain::Chain;
use super::endpoint::EndpointKind;
use sqlx::PgExecutor;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Peer {
    pub address: String,
    pub is_alive: Option<bool>, // TODO: make non-optional
    pub kind: EndpointKind,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub provider: String,
}

pub struct Peers(Vec<Peer>);

impl Peers {
    pub fn max_updated_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.0.iter().map(|p| p.updated_at).max()
    }

    pub fn persistent(&self) -> Vec<Peer> {
        self.0
            .iter()
            .filter_map(|p| {
                if p.kind == EndpointKind::Peer {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn seeds(&self) -> Vec<Peer> {
        self.0
            .iter()
            .filter_map(|p| {
                if p.kind == EndpointKind::Seed {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Chain {
    pub async fn peers(&self, executer: impl PgExecutor<'_>) -> sqlx::Result<Peers> {
        // TODO: determine is_alive differently
        // TODO: determine updated_at differently
        let kinds = vec![EndpointKind::Peer, EndpointKind::Seed];
        let peers = sqlx::query_as!(
            Peer,
            r#"
            SELECT address, true as is_alive, kind as "kind: EndpointKind", endpoint.created_at as updated_at, provider
            FROM endpoint
            INNER JOIN chain_endpoint ON endpoint.id = chain_endpoint.endpoint_id_fk
            WHERE chain_endpoint.chain_id_fk = $1 AND kind = ANY($2)
            "#,
            self.id,
            kinds as _
        )
        .fetch_all(executer)
        .await?;

        Ok(Peers(peers))
    }
}

pub async fn insert_persistent_peers(
    executor: impl PgExecutor<'_>,
    chain_id: &i64,
) -> sqlx::Result<Vec<i64>> {
    insert_peers(executor, chain_id, EndpointKind::Peer).await
}

pub async fn insert_seeds(executor: impl PgExecutor<'_>, chain_id: &i64) -> sqlx::Result<Vec<i64>> {
    insert_peers(executor, chain_id, EndpointKind::Seed).await
}

async fn insert_peers(
    executor: impl PgExecutor<'_>,
    chain_id: &i64,
    kind: EndpointKind,
) -> sqlx::Result<Vec<i64>> {
    let rows = sqlx::query!(
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

    Ok(rows.into_iter().map(|row| row.id).collect())
}

pub async fn join_chain_to_endpoints(
    executor: impl PgExecutor<'_>,
    chain_id: &i64,
    endpoint_ids: &Vec<i64>,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
    INSERT INTO chain_endpoint (chain_id_fk, endpoint_id_fk)
        SELECT $1, id FROM endpoint WHERE id = ANY($2)
        ON CONFLICT DO NOTHING
    "#,
        chain_id,
        &endpoint_ids,
    )
    .execute(executor)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    const CHAIN_ID: i64 = 1;

    #[sqlx::test(fixtures("chains"))]
    async fn test_insert_persistent_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let ids = insert_persistent_peers(&mut conn, &CHAIN_ID).await?;
        assert_eq!(ids.len(), 3);

        let seed = sqlx::query!(
            r#"
            SELECT distinct(kind)::text as kind FROM endpoint WHERE id = $1
            "#,
            ids[0],
        )
        .fetch_one(&mut conn)
        .await?;

        assert_eq!(seed.kind.unwrap(), "peer");

        // Tests ON CONFLICT
        let next_ids = insert_persistent_peers(&mut conn, &CHAIN_ID).await?;
        assert_eq!(ids, next_ids);

        join_chain_to_endpoints(&mut conn, &CHAIN_ID, &ids).await?;

        let rows = sqlx::query!(
            r#"
            SELECT endpoint_id_fk FROM chain_endpoint WHERE chain_id_fk = $1
            "#,
            CHAIN_ID,
        )
        .fetch_all(&mut conn)
        .await?;

        assert_eq!(rows.len(), 3);

        let join_ids: Vec<i64> = rows.into_iter().map(|row| row.endpoint_id_fk).collect();
        assert_eq!(ids, join_ids);

        Ok(())
    }

    #[sqlx::test(fixtures("chains"))]
    async fn test_insert_seeds(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let ids = insert_seeds(&mut conn, &CHAIN_ID).await?;
        assert_eq!(ids.len(), 7);

        let seed = sqlx::query!(
            r#"
            SELECT distinct(kind)::text as kind FROM endpoint WHERE id = $1
            "#,
            ids[0],
        )
        .fetch_one(&mut conn)
        .await?;

        assert_eq!(seed.kind.unwrap(), "seed");

        // Tests ON CONFLICT
        let next_ids = insert_seeds(&mut conn, &CHAIN_ID).await?;
        assert_eq!(ids, next_ids);

        join_chain_to_endpoints(&mut conn, &CHAIN_ID, &ids).await?;

        let rows = sqlx::query!(
            r#"
            SELECT endpoint_id_fk FROM chain_endpoint WHERE chain_id_fk = $1
            "#,
            CHAIN_ID,
        )
        .fetch_all(&mut conn)
        .await?;

        assert_eq!(rows.len(), 7);

        let join_ids: Vec<i64> = rows.into_iter().map(|row| row.endpoint_id_fk).collect();
        assert_eq!(ids, join_ids);

        Ok(())
    }

    #[sqlx::test(fixtures("peers"))]
    async fn test_chain_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let chain = Chain::new(&mut conn, &CHAIN_ID).await?;

        let peers = chain.peers(&mut conn).await?;
        assert_eq!(peers.0.len(), 4);

        let seeds = peers.seeds();
        assert_eq!(seeds.len(), 1);
        assert_eq!(seeds[0].kind, EndpointKind::Seed);
        assert_eq!(seeds[0].provider, "Company1");
        assert_eq!(seeds[0].address, "seed1@localhost:26656");

        let persistent = peers.persistent();
        assert_eq!(persistent.len(), 3);

        Ok(())
    }
}
