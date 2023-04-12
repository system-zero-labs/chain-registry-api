use crate::api::{from_db_error, internal_error, APIError, APIResponse, Meta};
use crate::db::peer::{
    filter_by_type, filter_recent_peers, find_commit, find_updated_at, PeerFilter, PeerType,
};
use axum::{extract::Path, extract::Query, extract::State, Json};
use sqlx::postgres::PgPool;

#[derive(Debug, serde::Serialize)]
pub struct PeerResult {
    pub seeds: Vec<String>,
    pub persistent: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PeerParams {
    include_all: bool,
}

pub async fn list_peers(
    State(pool): State<PgPool>,
    Path((network, chain_name)): Path<(String, String)>,
    params: Option<Query<PeerParams>>,
) -> Result<Json<APIResponse<PeerResult>>, APIError> {
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

    let resp = APIResponse {
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
