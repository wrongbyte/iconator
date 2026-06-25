use axum::response::IntoResponse;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Icon not found")]
    IconNotFound,
    #[error("Validation error: {0}")]
    ValidationError(String),
    // #[error("Internal server error: {0}")]
    // InternalServerError(String),
}

impl ApplicationError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            ApplicationError::IconNotFound => axum::http::StatusCode::NOT_FOUND,
            ApplicationError::ValidationError(_) => axum::http::StatusCode::BAD_REQUEST,
            // ApplicationError::InternalServerError(_) => {
            //     axum::http::StatusCode::INTERNAL_SERVER_ERROR
            // }
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