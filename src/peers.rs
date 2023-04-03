use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct RawPeer {
    node_id: Option<String>,
    address: Option<String>,
}

pub enum PeerType {
    Seed,
    Persistent,
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

pub async fn insert_peer<F: Fn(&str) -> anyhow::Result<()>>(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    chain_id: i64,
    peer_type: PeerType,
    peer: RawPeer,
    live_check: F,
) -> anyhow::Result<()> {
    let (node_id, address) = match (peer.node_id, peer.address) {
        (Some(node_id), Some(address)) => (node_id, address),
        _ => anyhow::bail!("peer is missing node_id or address"),
    };
    let is_alive = live_check(address.as_ref()).is_ok();
    let address = format!("{}@{}", node_id, address);

    match sqlx::query!(
        r#"
        INSERT INTO peer (chain_id_fk, address, type, is_alive)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (chain_id_fk, address, type) DO UPDATE SET is_alive = $4
        "#,
        chain_id,
        address,
        peer_type.as_str(),
        is_alive,
    )
    .execute(conn)
    .await
    {
        Ok(_) => Ok(()),
        Err(err) => anyhow::bail!("failed to insert peer: {:?}", err),
    }
}

pub fn tcp_check_liveness(addr: &str, timeout: Duration) -> anyhow::Result<()> {
    let socket_addrs = addr.to_socket_addrs()?;
    let mut last_error = None;

    for socket_addr in socket_addrs {
        match TcpStream::connect_timeout(&socket_addr, timeout) {
            Ok(stream) => {
                stream.shutdown(std::net::Shutdown::Both)?;
                return Ok(());
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "No good addresses",
    )))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::time::Duration;
    use tokio_test::*;

    #[test]
    fn test_tcp_check_liveness() {
        let timeout = Duration::from_secs(3);

        assert_err!(tcp_check_liveness("127.0.0.1:433", timeout));

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let local_addr = listener.local_addr().unwrap();
        let addr = format!("127.0.0.1:{}", local_addr.port());

        assert_ok!(tcp_check_liveness(addr.as_ref(), timeout));

        // Testing domain names
        assert_ok!(tcp_check_liveness("google.com:80", timeout));
    }

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

        let stub_liveness = |_: &str| -> anyhow::Result<()> { Ok(()) };

        let peer = RawPeer {
            node_id: Some("abc123".to_string()),
            address: Some("127.0.0.1:3346".to_string()),
        };

        assert_ok!(
            insert_peer(
                &mut conn,
                1,
                PeerType::Persistent,
                peer.clone(),
                stub_liveness
            )
            .await
        );

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
        assert_eq!(inserted.provider, "unknown");
        assert_eq!(inserted.r#type, "persistent");
        assert!(inserted.is_alive);

        let stub_liveness = |_: &str| -> anyhow::Result<()> { anyhow::bail!("boom") };
        assert_ok!(insert_peer(&mut conn, 1, PeerType::Persistent, peer, stub_liveness).await);

        let updated = sqlx::query!(
            r#"
            SELECT * FROM peer
            WHERE chain_id_fk = 1
            LIMIT 1
            "#,
        )
        .fetch_one(&mut conn)
        .await?;

        assert_eq!(inserted.id, updated.id);
        assert_eq!(inserted.address, updated.address);
        assert_eq!(inserted.chain_id_fk, updated.chain_id_fk);
        assert_eq!(inserted.provider, updated.provider);

        assert!(!updated.is_alive);

        Ok(())
    }
}
