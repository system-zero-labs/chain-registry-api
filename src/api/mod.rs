use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub(crate) mod chain;
pub(crate) mod peer;

#[derive(Debug, serde::Serialize)]
struct Meta {
    commit: String,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct APIResponse<T> {
    meta: Meta,
    result: T,
}

#[derive(Debug)]
pub enum APIError {
    NotFound,
    InternalServerError(String),
}

fn internal_error(err: impl std::fmt::Display) -> APIError {
    APIError::InternalServerError(err.to_string())
}

fn from_db_error(err: sqlx::Error) -> APIError {
    match err {
        sqlx::Error::RowNotFound => APIError::NotFound,
        _ => APIError::InternalServerError(err.to_string()),
    }
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            APIError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            // TODO: don't leak internal error messages to end user
            APIError::InternalServerError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
        };
        let body = json!({ "error": message });
        (status, Json(body)).into_response()
    }
}
