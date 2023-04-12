use serde::Deserialize;
use sqlx::PgExecutor;

#[derive(Debug, Clone, Deserialize)]
pub struct RawPeer {
    node_id: Option<String>,
    address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: i64,
    pub address: String,
    pub commit: String,
    pub is_alive: bool,
    pub peer_type: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerType {
    Seed,
    Persistent,
}

impl PeerType {
    pub fn from_str(s: &str) -> Option<PeerType> {
        match s {
            "seed" => Some(PeerType::Seed),
            "persistent" => Some(PeerType::Persistent),
            _ => None,
        }
    }

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
    executor: impl PgExecutor<'_>,
    chain_id: i64,
    peer_type: PeerType,
) -> sqlx::Result<Vec<RawPeer>> {
    sqlx::query_as!(
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
    .fetch_all(executor)
    .await
}

pub async fn insert_peer(
    executor: impl PgExecutor<'_>,
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
    .execute(executor)
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => anyhow::bail!(err),
    }
}

pub async fn all_recent_peers(executor: impl PgExecutor<'_>) -> sqlx::Result<Vec<Peer>> {
    sqlx::query_as!(
        Peer,
        r#"
        WITH recent_chain AS (
            SELECT commit, created_at FROM chain ORDER BY created_at DESC LIMIT 1
        )
        SELECT peer.id, address, peer.type as peer_type, chain.commit, peer.is_alive, peer.updated_at
        FROM peer INNER JOIN chain ON chain.id = peer.chain_id_fk WHERE chain.commit IN (SELECT commit FROM recent_chain)
        "#,
    )
        .fetch_all(executor)
        .await
}

pub async fn recent_peers(
    executor: impl PgExecutor<'_>,
    chain_name: &str,
    network: &str,
) -> sqlx::Result<Vec<Peer>> {
    sqlx::query_as!(
        Peer,
        r#"
        WITH recent_chain AS (
            SELECT commit, created_at FROM chain ORDER BY created_at DESC LIMIT 1
        )
        SELECT peer.id, peer.address, peer.type as peer_type, peer.is_alive, chain.commit, peer.updated_at
        FROM peer INNER JOIN chain ON chain.id = peer.chain_id_fk 
        WHERE chain.commit IN (SELECT commit FROM recent_chain) AND
        chain.name = $1 AND 
        chain.network = $2 
        "#,
        chain_name,
        network,
    )
    .fetch_all(executor)
    .await
}

pub struct PeerFilter {
    pub chain_name: String,
    pub network: String,
    pub peer_type: PeerType,
    pub is_alive: Option<bool>,
}

pub async fn filter_recent_peers(
    executor: impl PgExecutor<'_>,
    filter: &PeerFilter,
) -> sqlx::Result<Vec<String>> {
    let peers = recent_peers(executor, &filter.chain_name, &filter.network).await?;

    let filtered: Vec<String> = peers
        .into_iter()
        .filter(|p| PeerType::from_str(p.peer_type.as_str()) == Some(filter.peer_type.clone()))
        .filter(|p| {
            filter
                .is_alive
                .map(|alive| alive == p.is_alive)
                .unwrap_or(true)
        })
        .map(|p| p.address)
        .collect();

    if filtered.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(filtered)
}

pub async fn update_liveness<F: Fn(&str) -> anyhow::Result<()>>(
    executor: impl PgExecutor<'_>,
    peer: &Peer,
    check: F,
) -> sqlx::Result<()> {
    let alive = check(&peer.address).is_ok();
    sqlx::query!(
        r#"
        UPDATE peer SET is_alive = $1 WHERE id = $2
        "#,
        alive,
        peer.id,
    )
    .execute(executor)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use tokio_test::*;

    #[sqlx::test(fixtures("chains"))]
    async fn test_find_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let seeds = find_peers(&mut conn, 1, PeerType::Seed).await?;

        assert_eq!(seeds.len(), 7);

        for seed in seeds {
            assert!(seed.node_id.is_some());
            assert!(seed.address.is_some());
        }

        let peers = find_peers(&mut conn, 1, PeerType::Persistent).await?;

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
    async fn test_all_recent_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let found = all_recent_peers(&mut conn).await?;

        assert_eq!(found.len(), 2);

        Ok(())
    }

    #[sqlx::test(fixtures("recent_peers"))]
    async fn test_recent_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let found = recent_peers(&mut conn, "cosmoshub", "mainnet").await?;

        assert_eq!(found.len(), 2);

        let found = recent_peers(&mut conn, "juno", "mainnet").await?;

        assert_eq!(found.len(), 3);

        assert_eq!(found[0].address, "abc123@seed1.example.com");
        assert_eq!(found[0].commit, "new_commit");
        assert_eq!(found[0].peer_type, "seed");
        assert!(found[0].is_alive);

        Ok(())
    }

    #[sqlx::test(fixtures("recent_peers"))]
    async fn test_filter_recent_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let mut filter = PeerFilter {
            chain_name: "cosmoshub".to_string(),
            network: "mainnet".to_string(),
            peer_type: PeerType::Seed,
            is_alive: None,
        };
        let found = filter_recent_peers(&mut conn, &filter).await?;
        assert_eq!(vec!["abc123@public-seed-node.com:26656"], found);

        filter.peer_type = PeerType::Persistent;
        let found = filter_recent_peers(&mut conn, &filter).await?;
        assert_eq!(vec!["efg987@public-persistent.com:26656"], found);

        filter.is_alive = Some(true);
        assert_err!(filter_recent_peers(&mut conn, &filter).await);

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
            commit: "stub".to_string(),
            peer_type: "seed".to_string(),
            is_alive: true,
            updated_at: chrono::Utc::now(),
        };

        update_liveness(&mut conn, &peer, stub_liveness).await?;

        let updated = sqlx::query!(
            r#"
            SELECT is_alive FROM peer WHERE id = 1
            "#,
        )
        .fetch_one(&mut conn)
        .await?;

        assert!(!updated.is_alive);

        let stub_liveness = |_: &str| -> anyhow::Result<()> { Ok(()) };
        update_liveness(&mut conn, &peer, stub_liveness).await?;

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
