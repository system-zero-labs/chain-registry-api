use crate::api::{from_db_error, internal_error, APIError};
use crate::db::chain;
use axum::{extract::Path, extract::State, Json};
use sqlx::postgres::PgPool;

pub async fn get_chain_data(
    State(pool): State<PgPool>,
    Path((chain_name, network)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let chain_data = chain::find_chain(&mut conn, chain_name.as_str(), network.as_str())
        .await
        .map_err(from_db_error)?;

    Ok(Json(chain_data.chain_data))
}
