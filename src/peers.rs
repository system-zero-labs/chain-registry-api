use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawPeer {
    node_id: String,
    address: String,
    provider: Option<String>,
}

pub async fn insert_peer<F: Fn(&str) -> anyhow::Result<()>>(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    chain_id: i64,
    peer_type: String, // TODO: should be an enum
    live_check: F,
) -> anyhow::Result<()> {
    anyhow::bail!("Not implemented");
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
    use super::{insert_peer, tcp_check_liveness};
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

        // insert_persistent_peer(&mut conn, 1).await.unwrap();
        panic!("TODO");

        Ok(())
    }
}
