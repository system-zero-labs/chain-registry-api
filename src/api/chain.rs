use crate::api::{from_db_error, internal_error, APIError, APIResponse, Meta};
use crate::db::chain;
use axum::{extract::Path, extract::State, Json};
use sqlx::postgres::PgPool;

pub async fn get_chain_data(
    State(pool): State<PgPool>,
    Path((chain_name, network)): Path<(String, String)>,
) -> Result<Json<APIResponse<serde_json::Value>>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let chain = chain::find_chain(&mut conn, chain_name.as_str(), network.as_str())
        .await
        .map_err(from_db_error)?;

    let resp = APIResponse {
        meta: Meta {
            commit: chain.commit,
            updated_at: chain.updated_at,
        },
        result: chain.chain_data,
    };

    Ok(Json(resp))
}

pub async fn get_chain_asset_list(
    State(pool): State<PgPool>,
    Path((chain_name, network)): Path<(String, String)>,
) -> Result<Json<APIResponse<serde_json::Value>>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let chain = chain::find_chain(&mut conn, chain_name.as_str(), network.as_str())
        .await
        .map_err(from_db_error)?;

    let resp = APIResponse {
        meta: Meta {
            commit: chain.commit,
            updated_at: chain.updated_at,
        },
        result: chain.asset_data,
    };

    Ok(Json(resp))
}

pub async fn list_chains(
    State(pool): State<PgPool>,
    Path(network): Path<String>,
) -> Result<Json<APIResponse<Vec<String>>>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let list = chain::list_chains(&mut conn, network.as_str())
        .await
        .map_err(from_db_error)?;

    let resp = APIResponse {
        meta: Meta {
            commit: list.commit,
            updated_at: list.updated_at,
        },
        result: list.names,
    };
    Ok(Json(resp))
}
