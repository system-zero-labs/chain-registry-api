use crate::api::{from_db_error, internal_error, APIError, APIResponse, Meta};
use crate::db::peer::{PeerType, PeerFilter, filter_recent_peers};
use axum::{extract::Path, extract::State, extract::Query, Json};
use sqlx::postgres::PgPool;

#[derive(Debug, serde::Serialize)]
pub struct PeerResult {
    pub seeds: Vec<String>,
    pub persistent: Vec<String>,
}

pub struct PeerParams {
    pub filter: String
}

pub async fn list_peers(
    State(pool): State<PgPool>,
    Path((network, chain_name)): Path<(String, String)>,
    Query(params): Query<PeerParams>,
) -> Result<Json<APIResponse<PeerResult>>, APIError> {
    let is_alive = match params.filter.as_str() {
        "all" => None,
        _ => Some(true),
    };
    let filter = PeerFilter {
        chain_name,
        network,
        peer_type: PeerType::Seed,
        is_alive,
    };

    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let seeds = filter_recent_peers()

    let resp = APIResponse {
        meta: Meta { commit, updated_at },
        result,
    };

    Ok(Json(resp))
}
