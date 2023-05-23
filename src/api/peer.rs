use crate::api::{from_db_error, internal_error, APIError, Meta};
use crate::db::chain::{Chain, Network};
use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct PeerList {
    meta: Meta,
    result: PeerResult,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Peer {
    address: String,
    last_liveness_check: chrono::DateTime<chrono::Utc>,
    provider: String,
    is_alive: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PeerResult {
    pub seeds: Vec<Peer>,
    pub persistent: Vec<Peer>,
}

#[derive(Debug, Deserialize)]
pub struct PeerParams {
    include_all: bool,
}

/// Get chain's live seeds and persistent peers.
/// A background process periodically checks peers for liveness. If a peer cannot be reached,
/// it is excluded from this response by default.
#[utoipa::path(
get,
path = "/v1/{network}/{chain_name}/peers",
responses(
(status = 200, description = "Peers found successfully", body = PeerList),
(status = 404, description = "Network or chain does not exist, or chain does not have any peers"),
),
params(
("network" = String, Path, description = "mainnet or testnet"),
("chain_name" = String, Path, description = "Chain name, e.g. cosmoshub"),
("include_all" = Option<bool>, Query, description = "If true, include all peers regardless of liveness"),
),
tag = "Peers",
)]
pub async fn list_peers(
    State(pool): State<PgPool>,
    Path((network, chain_name)): Path<(String, String)>,
    // TODO: use params
    params: Option<Query<PeerParams>>,
) -> Result<Json<PeerList>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    // TODO: DRY up network handling in middleware
    let network =
        Network::from_str(network.as_str()).map_err(|err| APIError::BadRequest(err.to_string()))?;
    let chain = Chain::from_name(&mut conn, chain_name.as_str(), &network)
        .await
        .map_err(from_db_error)?;
    let peers = chain.peers(&mut conn).await.map_err(from_db_error)?;

    let to_peers = |peers: Vec<crate::db::peer::Peer>| -> Vec<Peer> {
        peers
            .into_iter()
            .map(|p| Peer {
                address: p.address,
                provider: p.provider,
                last_liveness_check: p.updated_at, // TODO: fix
                is_alive: true,                    // TODO: fix
            })
            .collect()
    };
    let resp = PeerList {
        meta: Meta {
            commit: chain.commit,
            updated_at: peers.max_updated_at().unwrap_or(chain.created_at),
        },
        result: PeerResult {
            seeds: to_peers(peers.seeds()),
            persistent: to_peers(peers.persistent()),
        },
    };

    Ok(Json(resp))
}

/// Get a chain's live seeds as a comma-separated string for use in config.toml.
/// A background process periodically checks seeds for liveness. If a seed cannot be reached,
/// it is excluded from this response by default.
#[utoipa::path(
get,
path = "/v1/{network}/{chain_name}/peers/seed_string",
responses(
(status = 200, description = "Seeds found successfully", body = String),
(status = 404, description = "Network or chain does not exist, or chain does not have any seeds"),
),
params(
("network" = String, Path, description = "mainnet or testnet"),
("chain_name" = String, Path, description = "Chain name, e.g. cosmoshub"),
("include_all" = Option<bool>, Query, description = "If true, include all seeds regardless of liveness"),
),
tag = "Peers",
)]
pub async fn seed_string(
    State(pool): State<PgPool>,
    Path((network, chain_name)): Path<(String, String)>,
    params: Option<Query<PeerParams>>,
) -> Result<String, APIError> {
    let seeds = list_peers(State(pool), Path((network, chain_name)), params)
        .await?
        .0
        .result;

    let seeds = seeds
        .seeds
        .into_iter()
        .map(|p| p.address)
        .collect::<Vec<String>>();

    Ok(seeds.join(","))
}

/// Get a chain's live persistent peers as a comma-separated string for use in config.toml.
/// A background process periodically checks peers for liveness. If a peer cannot be reached,
/// it is excluded from this response by default.
#[utoipa::path(
get,
path = "/v1/{network}/{chain_name}/peers/peer_string",
responses(
(status = 200, description = "Peers found successfully", body = String),
(status = 404, description = "Network or chain does not exist, or chain does not have any persistent peers"),
),
params(
("network" = String, Path, description = "mainnet or testnet"),
("chain_name" = String, Path, description = "Chain name, e.g. cosmoshub"),
("include_all" = Option<bool>, Query, description = "If true, include all peers regardless of liveness"),
),
tag = "Peers",
)]
pub async fn persistent_peer_string(
    State(pool): State<PgPool>,
    Path((network, chain_name)): Path<(String, String)>,
    params: Option<Query<PeerParams>>,
) -> Result<String, APIError> {
    let peers = list_peers(State(pool), Path((network, chain_name)), params)
        .await?
        .0
        .result;

    let peers = peers
        .persistent
        .into_iter()
        .map(|p| p.address)
        .collect::<Vec<String>>();

    Ok(peers.join(","))
}
