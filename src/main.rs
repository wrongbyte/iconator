use axum::{
    Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use http::header;
use iconator::{get_icon_for_file, get_icon_for_folder};
use serde::Deserialize;
use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::ApplicationError;

mod error;

// TODO: not sure if we need a state
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
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("could not start server");
}

pub fn router() -> Router<AppState> {
    Router::new().route("/icon/", get(get_icon_id))
}

//  TYPES -------------------------------------------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    File,
    Folder,
}

#[derive(Debug, Deserialize)]
struct IconQuery {
    pub r#type: FileType,
    pub path: String,
}

//   -------------------------------------------------------------------
#[tracing::instrument(skip(_state))]
async fn get_icon_id(
    State(_state): State<AppState>,
    Query(params): Query<IconQuery>,
) -> impl IntoResponse {
    let IconQuery { r#type, path } = params;
    let path = Path::new(&path); 
    println!("path: {:?}, is_file: {}, is_dir: {}", path, path.is_file(), path.is_dir());

    let icon_id = match r#type {
        FileType::File if path.is_file() => get_icon_for_file(path),
        FileType::Folder if path.is_dir() => get_icon_for_folder(path),
        _ => {
            return Err(ApplicationError::ValidationError(
                format!("invalid target or path: {}", path.display()),
            ));
        }
    }
    .ok_or(ApplicationError::IconNotFound)?;

    let svg_path = std::path::Path::new("/icons").join(format!("{icon_id}.svg"));
    let svg = tokio::fs::read_to_string(&svg_path)
        .await
        .map_err(|_| ApplicationError::IconNotFound)?;

    Ok(([(header::CONTENT_TYPE, "image/svg+xml")], svg))
}
