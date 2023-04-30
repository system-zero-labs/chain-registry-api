use crate::api::{from_db_error, internal_error, APIError, APIResponse, Meta};
use crate::db::chain;
use axum::{extract::Path, extract::State, Json};
use serde::Serialize;
use sqlx::postgres::PgPool;
use utoipa::ToSchema;

/// Get chain's data.
///
/// Fetches all metadata for a chain, such as the binary, bech32 prefix, genesis file, peers, rpc endpoints, etc.
/// Currently, this OpenAPI spec does not include a schema because it may change without warning.
/// The response should match the schema at: https://github.com/cosmos/chain-registry/blob/master/chain.schema.json.
#[utoipa::path(
get,
path = "/v1/{network}/{chain_name}",
responses(
(status = 200, description = "Chain found successfully"),
(status = 404, description = "Network or chain does not exist"),
),
params(
("network" = String, Path, description = "mainnet or testnet"),
("chain_name" = String, Path, description = "Chain name, e.g. cosmoshub")
),
tag = "Chains",
)]
pub async fn get_chain_data(
    State(pool): State<PgPool>,
    Path((network, chain_name)): Path<(String, String)>,
) -> Result<Json<APIResponse<serde_json::Value>>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let chain = chain::find_chain(&mut conn, network.as_str(), chain_name.as_str())
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

/// Get chain's assetlist.
///
/// Asset lists allow frontends and other UIs to fetch metadata associated with Cosmos SDK denoms, especially for assets sent over IBC.
/// Currently, this OpenAPI spec does not include a schema because it may change without warning.
/// The response should match the schema at: https://github.com/cosmos/chain-registry/blob/master/assetlist.schema.json
#[utoipa::path(
get,
path = "/v1/{network}/{chain_name}/assetlist",
responses(
(status = 200, description = "Assetlist found successfully"),
(status = 404, description = "Network, chain, or assetlist does not exist"),
),
params(
("network" = String, Path, description = "mainnet or testnet"),
("chain_name" = String, Path, description = "Chain name, e.g. cosmoshub")
),
tag = "Chains",
)]
pub async fn get_chain_asset_list(
    State(pool): State<PgPool>,
    Path((network, chain_name)): Path<(String, String)>,
) -> Result<Json<APIResponse<serde_json::Value>>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let chain = chain::find_chain(&mut conn, network.as_str(), chain_name.as_str())
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

#[derive(Debug, Serialize, ToSchema)]
pub struct ChainList {
    meta: Meta,
    result: Vec<String>,
}

/// List chains by network
#[utoipa::path(
get,
path = "/v1/{network}/chains",
responses(
(status = 200, description = "Chains found successfully", body = ChainList),
(status = 404, description = "Network does not exist"),
),
params(
("network" = String, Path, description = "mainnet or testnet")
),
tag = "Chains",
)]
pub async fn list_chains(
    State(pool): State<PgPool>,
    Path(network): Path<String>,
) -> Result<Json<ChainList>, APIError> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    let list = chain::list_chains(&mut conn, network.as_str())
        .await
        .map_err(from_db_error)?;

    let resp = ChainList {
        meta: Meta {
            commit: list.commit,
            updated_at: list.updated_at,
        },
        result: list.names,
    };
    Ok(Json(resp))
}
