use axum::{
    Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use http::header;
use iconator::{get_icon_for_file, get_icon_for_folder};
use serde::Deserialize;
use tracing::warn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::ApplicationError;

mod error;

// The state is empty for now, but we can add fields later if needed
#[derive(Debug, Clone)]
pub struct AppState {}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = router().with_state(AppState {});

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("could not start server");
}

pub fn router() -> Router<AppState> {
    Router::new().route("/icon/", get(get_icon_id))
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PathType {
    File,
    Folder,
}

#[derive(Debug, Deserialize)]
struct IconQuery {
    pub r#type: PathType,
    pub path: String,
}

#[tracing::instrument(skip(_state))]
async fn get_icon_id(
    State(_state): State<AppState>,
    Query(params): Query<IconQuery>,
) -> Result<impl IntoResponse, ApplicationError> {
    let IconQuery { r#type, path } = params;
    let icon_id = match r#type {
        PathType::File => get_icon_for_file(&path).map_err(|e| ApplicationError::PathError(e)),
        PathType::Folder => get_icon_for_folder(&path).map_err(|e| ApplicationError::PathError(e)),
    }?
    .ok_or(ApplicationError::IconNotFound(path))?;

    let svg_path = std::path::Path::new("icons").join(format!("{icon_id}.svg"));

    let svg = tokio::fs::read_to_string(&svg_path).await.map_err(|e| {
        warn!("failed to read SVG file {}: {:?}", svg_path.display(), e);
        ApplicationError::InternalError
    })?;

    Ok(([(header::CONTENT_TYPE, "image/svg+xml")], svg))
}
