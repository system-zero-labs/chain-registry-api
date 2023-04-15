use crate::api::{from_db_error, internal_error, APIError, Meta};
use crate::db::peer::{
    filter_by_type, filter_recent_peers, find_commit, find_updated_at, PeerFilter, PeerType,
};
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
pub struct PeerResult {
    pub seeds: Vec<String>,
    pub persistent: Vec<String>,
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
    params: Option<Query<PeerParams>>,
) -> Result<Json<PeerList>, APIError> {
    let include_all = params.map(|p| p.include_all).unwrap_or(false);
    let filter = PeerFilter {
        chain_name,
        network,
        include_all,
    };

    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let peers = filter_recent_peers(&mut conn, &filter)
        .await
        .map_err(from_db_error)?;

    let commit = find_commit(&peers).unwrap_or_default();
    let updated_at = find_updated_at(&peers).unwrap_or_default();

    let resp = PeerList {
        meta: Meta { commit, updated_at },
        result: PeerResult {
            seeds: filter_by_type(&peers, PeerType::Seed)
                .iter()
                .map(|p| p.address.clone())
                .collect(),
            persistent: filter_by_type(&peers, PeerType::Persistent)
                .iter()
                .map(|p| p.address.clone())
                .collect(),
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
    let resp = list_peers(State(pool), Path((network, chain_name)), params)
        .await?
        .0
        .result;

    Ok(resp.seeds.join(","))
}

/// Get a chain's live persistent peers as a comma-separated string for use in config.toml.
/// A background process periodically checks peers for liveness. If a peer cannot be reached,
/// it is excluded from this response by default.
#[utoipa::path(
get,
path = "/v1/{network}/{chain_name}/peers/persistent_peer_string",
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
    let resp = list_peers(State(pool), Path((network, chain_name)), params)
        .await?
        .0
        .result;

    Ok(resp.persistent.join(","))
}
