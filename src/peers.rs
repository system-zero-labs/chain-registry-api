use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawPeer {
    node_id: Option<String>,
    address: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PeerType {
    Seed,
    Persistent,
}

#[derive(Debug, Clone)]
pub struct Peer {
    id: i64,
    address: String,
}

impl PeerType {
    fn as_field(&self) -> &str {
        match self {
            PeerType::Seed => "seeds",
            PeerType::Persistent => "persistent_peers",
        }
    }

    fn as_str(&self) -> &str {
        match self {
            PeerType::Seed => "seed",
            PeerType::Persistent => "persistent",
        }
    }
}

pub async fn find_peers(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    chain_id: i64,
    peer_type: PeerType,
) -> anyhow::Result<Vec<RawPeer>> {
    match sqlx::query_as!(
        RawPeer,
        r#"
        select 
        jsonb_array_elements(chain_data->'peers'->$1)->>'id' as node_id, 
        jsonb_array_elements(chain_data->'peers'->$1)->>'address' as address
        from chain where id = $2
        "#,
        peer_type.as_field(),
        chain_id,
    )
    .fetch_all(conn)
    .await
    {
        Ok(peers) => Ok(peers),
        Err(err) => anyhow::bail!("failed to find {} peers: {:?}", peer_type.as_field(), err),
    }
}

pub async fn insert_peer(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    chain_id: i64,
    peer_type: PeerType,
    peer: RawPeer,
) -> anyhow::Result<()> {
    let (node_id, address) = match (peer.node_id, peer.address) {
        (Some(node_id), Some(address)) => (node_id, address),
        _ => anyhow::bail!("peer is missing node_id or address"),
    };
    let address = format!("{}@{}", node_id, address);

    // The bogus DO UPDATE SET ensures we don't get a RowNotFound error.
    match sqlx::query!(
        r#"
        INSERT INTO peer (chain_id_fk, address, type)
        VALUES ($1, $2, $3)
        ON CONFLICT (chain_id_fk, address, type) DO UPDATE SET is_alive = peer.is_alive
        "#,
        chain_id,
        address,
        peer_type.as_str(),
    )
    .execute(conn)
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => anyhow::bail!("failed to insert peer: {:?}", err),
    }
}

pub async fn recent_peers(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
) -> anyhow::Result<Vec<Peer>> {
    match sqlx::query_as!(
        Peer,
        r#"
        WITH recent_chain AS (
        SELECT commit, max(created_at) as created_at FROM chain GROUP BY commit ORDER BY created_at DESC LIMIT 1
        )
        SELECT peer.id, address FROM peer JOIN chain ON chain.id = peer.chain_id_fk WHERE chain.commit IN (SELECT commit FROM recent_chain)
        "#,
    )
        .fetch_all(conn)
        .await
    {
        Ok(peers) => Ok(peers),
        Err(err) => anyhow::bail!("failed to find recent peers: {:?}", err),
    }
}

pub async fn update_liveness<F: Fn(&str) -> anyhow::Result<()>>(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    peer: &Peer,
    check: F,
) -> anyhow::Result<()> {
    let alive = check(&peer.address).is_ok();
    match sqlx::query!(
        r#"
        UPDATE peer SET is_alive = $1 WHERE id = $2
        "#,
        alive,
        peer.id,
    )
    .execute(conn)
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => anyhow::bail!("failed to update peer {} liveness: {:?}", peer.address, err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use tokio_test::*;

    #[sqlx::test(fixtures("chains"))]
    async fn test_find_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let seeds = find_peers(&mut conn, 1, PeerType::Seed).await.unwrap();

        assert_eq!(seeds.len(), 7);

        for seed in seeds {
            assert!(seed.node_id.is_some());
            assert!(seed.address.is_some());
        }

        let peers = find_peers(&mut conn, 1, PeerType::Persistent)
            .await
            .unwrap();

        assert_eq!(peers.len(), 3);

        for peer in peers {
            assert!(peer.node_id.is_some());
            assert!(peer.address.is_some());
        }

        Ok(())
    }

    #[sqlx::test(fixtures("chains"))]
    async fn test_insert_persistent_peer(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let peer = RawPeer {
            node_id: Some("abc123".to_string()),
            address: Some("127.0.0.1:3346".to_string()),
        };

        assert_ok!(insert_peer(&mut conn, 1, PeerType::Persistent, peer.clone(),).await);

        let inserted = sqlx::query!(
            r#"
            SELECT * FROM peer
            WHERE chain_id_fk = 1
            LIMIT 1
            "#,
        )
        .fetch_one(&mut conn)
        .await?;

        assert_eq!(inserted.address, "abc123@127.0.0.1:3346");
        assert_eq!(inserted.chain_id_fk, 1);
        assert_eq!(inserted.r#type, "persistent");
        assert!(inserted.is_alive);

        Ok(())
    }

    #[sqlx::test(fixtures("recent_peers"))]
    async fn test_recent_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let found = recent_peers(&mut conn).await.unwrap();

        assert_eq!(found.len(), 2);

        Ok(())
    }

    #[sqlx::test(fixtures("recent_peers"))]
    async fn test_update_liveness(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let stub_liveness = |addr: &str| -> anyhow::Result<()> {
            assert_eq!(addr, "stub@address");
            anyhow::bail!("boom")
        };

        let peer = Peer {
            id: 1,
            address: "stub@address".to_string(),
        };

        update_liveness(&mut conn, &peer, stub_liveness)
            .await
            .unwrap();

        let updated = sqlx::query!(
            r#"
            SELECT is_alive FROM peer WHERE id = 1
            "#,
        )
        .fetch_one(&mut conn)
        .await?;

        assert!(!updated.is_alive);

        let stub_liveness = |_: &str| -> anyhow::Result<()> { Ok(()) };
        update_liveness(&mut conn, &peer, stub_liveness)
            .await
            .unwrap();

        let updated = sqlx::query!(
            r#"
            SELECT is_alive FROM peer WHERE id = 1
            "#,
        )
        .fetch_one(&mut conn)
        .await?;

        assert!(updated.is_alive);

        Ok(())
    }
}
