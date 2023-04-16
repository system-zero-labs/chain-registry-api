use axum::Router;
use tower_http::services::ServeDir;

pub fn static_web() -> Router {
    Router::new().nest_service("/", ServeDir::new("static"))
}
