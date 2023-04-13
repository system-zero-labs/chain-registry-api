use axum::{routing::get, Router};

use crate::api::chain::{get_chain_asset_list, get_chain_data, list_chains};
use crate::api::peer::{list_peers, persistent_peer_string, seed_string};

pub fn new() -> Router<sqlx::postgres::PgPool> {
    Router::new()
        .route("/:network", get(list_chains))
        .route("/:network/chains", get(list_chains))
        .route("/:network/:chain_name", get(get_chain_data))
        .route("/:network/:chain_name/assetlist", get(get_chain_asset_list))
        .route("/:network/:chain_name/peers", get(list_peers))
        .route("/:network/:chain_name/peers/seed_string", get(seed_string))
        .route(
            "/:network/:chain_name/peers/persistent_peer_string",
            get(persistent_peer_string),
        )
}
