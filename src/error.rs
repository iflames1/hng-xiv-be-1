use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    UnprocessableEntity(String),
    NotFound(String),
    Upstream(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    status: &'static str,
    message: String,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn unprocessable_entity(message: impl Into<String>) -> Self {
        Self::UnprocessableEntity(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn upstream(api: &str) -> Self {
        Self::Upstream(format!("{api} returned an invalid response"))
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::UnprocessableEntity(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Self::Upstream(message) => (StatusCode::BAD_GATEWAY, message),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        (
            status,
            Json(ErrorBody {
                status: "error",
                message,
            }),
        )
            .into_response()
    }
}
