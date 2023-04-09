use crate::db::chain;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use sqlx::postgres::PgPool;

pub async fn get_chain_data(
    State(pool): State<PgPool>,
    Path((chain_name, network)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    // TODO: error handling
    let mut conn = pool.acquire().await.unwrap();
    let chain_data = chain::find_chain(&mut conn, chain_name.as_str(), network.as_str())
        .await
        .unwrap();

    Json(chain_data.chain_data)
}
