use crate::api::{from_db_error, internal_error, APIError, APIResponse, Meta};
use crate::db::peer::{recent_peers, PeerType};
use axum::{extract::Path, extract::State, Json};
use sqlx::postgres::PgPool;

#[derive(Debug, serde::Serialize)]
pub struct PeerResult {
    pub seeds: Vec<String>,
    pub persistent: Vec<String>,
}

pub async fn list_peers(
    State(pool): State<PgPool>,
    Path((network, chain_name)): Path<(String, String)>,
) -> Result<Json<APIResponse<PeerResult>>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let peers = recent_peers(&mut conn, chain_name.as_str(), network.as_str())
        .await
        .map_err(from_db_error)?;

    let commit = peers.first().map(|p| p.commit.clone()).unwrap_or_default();
    let updated_at = peers.iter().map(|p| p.updated_at).max().unwrap_or_default();

    let result = PeerResult {
        seeds: peers
            .iter()
            .filter(|p| PeerType::from_str(p.peer_type.as_str()) == Some(PeerType::Seed))
            .map(|p| &p.address)
            .cloned()
            .collect(),
        persistent: peers
            .into_iter()
            .filter(|p| PeerType::from_str(p.peer_type.as_str()) == Some(PeerType::Persistent))
            .map(|p| p.address)
            .collect(),
    };

    let resp = APIResponse {
        meta: Meta { commit, updated_at },
        result,
    };

    Ok(Json(resp))
}
