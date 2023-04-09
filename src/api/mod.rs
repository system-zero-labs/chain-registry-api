use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub(crate) mod chain;

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
            APIError::InternalServerError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
        };
        let body = json!({ "error": message });
        (status, Json(body)).into_response()
    }
}
