use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde_json::Value;

use crate::{state::AppState, system::service};

async fn get_health(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let health = service::get_health(&state).await?;
    Ok(health)
}

async fn get_version() -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let version = service::get_version().await?;
    Ok(version)
}

pub fn system_routes() -> Router<AppState> {
    let public = Router::new()
        .route("/health", get(get_health))
        .route("/version", get(get_version));

    Router::new().nest("/system", public)
}
