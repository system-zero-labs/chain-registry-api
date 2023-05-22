use sqlx::PgExecutor;

#[derive(Debug, Clone)]
pub struct PeerDeprecated {
    pub id: i64,
    pub address: String,
    pub commit: String,
    pub is_alive: bool,
    pub peer_type: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub type PeersDeprecated = Vec<PeerDeprecated>;

pub fn find_commit(peers: &PeersDeprecated) -> Option<String> {
    peers.first().map(|p| p.commit.clone())
}

pub fn find_updated_at(peers: &PeersDeprecated) -> Option<chrono::DateTime<chrono::Utc>> {
    peers.iter().map(|p| p.updated_at.clone()).max()
}

pub fn filter_by_type(peers: &PeersDeprecated, peer_type: PeerType) -> PeersDeprecated {
    peers
        .iter()
        .filter(|p| PeerType::from_str(p.peer_type.as_str()) == Some(peer_type))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Copy)]
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
}

pub async fn all_recent_peers(executor: impl PgExecutor<'_>) -> sqlx::Result<PeersDeprecated> {
    sqlx::query_as!(
        PeerDeprecated,
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
) -> sqlx::Result<Vec<PeerDeprecated>> {
    sqlx::query_as!(
        PeerDeprecated,
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
    pub include_all: bool,
}

pub async fn filter_recent_peers(
    executor: impl PgExecutor<'_>,
    filter: &PeerFilter,
) -> sqlx::Result<PeersDeprecated> {
    let peers = recent_peers(executor, &filter.chain_name, &filter.network).await?;

    let filtered: Vec<PeerDeprecated> = peers
        .into_iter()
        .filter(|p| {
            if filter.include_all {
                return true;
            }
            p.is_alive
        })
        .collect();

    if filtered.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(filtered)
}

pub async fn update_liveness<F: Fn(&str) -> anyhow::Result<()>>(
    executor: impl PgExecutor<'_>,
    peer: &PeerDeprecated,
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

    #[sqlx::test(fixtures("recent_peers"))]
    async fn test_all_recent_peers(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;
        let found = all_recent_peers(&mut conn).await?;

        assert_eq!(found.len(), 5);

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
            include_all: true,
        };
        let found = filter_recent_peers(&mut conn, &filter).await?;
        assert_eq!(found.len(), 2);
        assert_eq!("abc123@public-seed-node.com:26656", found[0].address);
        assert_eq!("efg987@public-persistent.com:26656", found[1].address);

        filter.include_all = false;
        let found = filter_recent_peers(&mut conn, &filter).await?;
        assert_eq!(found.len(), 1);

        Ok(())
    }

    #[sqlx::test(fixtures("recent_peers"))]
    async fn test_update_liveness(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let stub_liveness = |addr: &str| -> anyhow::Result<()> {
            assert_eq!(addr, "stub@address");
            anyhow::bail!("boom")
        };

        let peer = PeerDeprecated {
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
