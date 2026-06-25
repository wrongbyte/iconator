use axum::response::IntoResponse;
use iconator::IconatorError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Icon not found for path {0}")]
    IconNotFound(String),
    #[error("{0}")]
    PathError(IconatorError),
    #[error("Internal error")]
    InternalError,
}

impl ApplicationError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            ApplicationError::IconNotFound(_) => axum::http::StatusCode::NOT_FOUND,
            ApplicationError::PathError(_) => axum::http::StatusCode::BAD_REQUEST,
            ApplicationError::InternalError => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status_code();
        let body = axum::Json(serde_json::json!({
            "error": self.to_string(),
        }));
        (status_code, body).into_response()
    }
}
