use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::chain::{get_chain_asset_list, get_chain_data, list_chains};
use crate::api::peer::{list_peers, persistent_peer_string, seed_string};

#[derive(OpenApi)]
#[openapi(paths(
    crate::api::chain::list_chains,
    crate::api::chain::get_chain_data,
    crate::api::chain::get_chain_asset_list,
))]
// components(
// schemas(APIResponse<Vec<String>>, Meta)
// ),
//
struct ApiDoc;

pub fn new() -> Router<sqlx::postgres::PgPool> {
    let v1_routes = Router::new()
        .route("/:network/chains", get(list_chains))
        .route("/:network/:chain_name", get(get_chain_data))
        .route("/:network/:chain_name/assetlist", get(get_chain_asset_list))
        .route("/:network/:chain_name/peers", get(list_peers))
        .route("/:network/:chain_name/peers/seed_string", get(seed_string))
        .route(
            "/:network/:chain_name/peers/persistent_peer_string",
            get(persistent_peer_string),
        );

    let mut doc = ApiDoc::openapi();
    doc.info.title = String::from("Chain Registry API");

    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", doc))
        .nest("/v1", v1_routes)
}
