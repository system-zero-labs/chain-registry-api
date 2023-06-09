use crate::api::chain::{
    get_chain_asset_list, get_chain_data, list_chains, ChainList, ChainListItem,
};
use crate::api::peer::{
    list_peers, persistent_peer_string, seed_string, Peer, PeerList, PeerResult,
};
use crate::api::Meta;
use axum::{routing::get, Router};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::chain::get_chain_asset_list,
        crate::api::chain::get_chain_data,
        crate::api::chain::list_chains,
        crate::api::peer::list_peers,
        crate::api::peer::persistent_peer_string,
        crate::api::peer::seed_string,
    ),
    components(schemas(Peer, PeerList, PeerResult, Meta, ChainList, ChainListItem))
)]
struct ApiDoc;

pub fn new() -> Router<sqlx::postgres::PgPool> {
    let v1_routes = Router::new()
        .route("/:network/chains", get(list_chains))
        .route("/:network/:chain_name", get(get_chain_data))
        .route("/:network/:chain_name/assetlist", get(get_chain_asset_list))
        .route("/:network/:chain_name/peers", get(list_peers))
        .route("/:network/:chain_name/peers/seed_string", get(seed_string))
        .route(
            "/:network/:chain_name/peers/peer_string",
            get(persistent_peer_string),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let mut doc = ApiDoc::openapi();
    doc.info.title = String::from("Chain Registry API");
    let license = utoipa::openapi::LicenseBuilder::new()
        .name("MIT License")
        .url(Some(
            "https://github.com/system-zero-labs/chain-registry-api/blob/main/LICENSE".to_string(),
        ))
        .build();
    doc.info.license = Some(license);

    Router::new()
        .merge(SwaggerUi::new("/v1-docs").url("/v1-api-docs/openapi.json", doc))
        .nest("/v1", v1_routes)
}
