use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPeer {
    chain_id: i64, // foreign key
    node_id: String,
    address: String,
    provider: Option<String>,
    peer_type: String, // TODO: should be enum, only 'persistent' and 'seed' are valid
}

pub async fn insert_peer<F: Fn(&str) -> anyhow::Result<()>>(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    peer: RawPeer,
    live_check: F,
) -> anyhow::Result<()> {
    let is_alive = live_check(peer.address.as_ref()).is_ok();
    let address = format!("{}@{}", peer.node_id, peer.address);

    match sqlx::query!(
        r#"
        INSERT INTO peer (chain_id_fk, address, provider, type, is_alive)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        peer.chain_id,
        address,
        peer.provider.unwrap_or("unknown".to_string()),
        peer.peer_type,
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
    async fn test_insert_persistent_peer(pool: PgPool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let stub_liveness = |_: &str| -> anyhow::Result<()> { Ok(()) };

        let peer = RawPeer {
            chain_id: 1,
            node_id: "abc123".to_string(),
            peer_type: "persistent".to_string(),
            address: "127.0.0.1:3346".to_string(),
            provider: None,
        };

        assert_ok!(insert_peer(&mut conn, peer, stub_liveness).await);

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

        Ok(())
    }
}
