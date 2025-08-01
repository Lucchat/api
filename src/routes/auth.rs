use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    middleware,
    routing::{get, post},
    Json, Router,
};

use crate::{
    auth::{
        jwt::require_refresh_token,
        model::{LoginPayload, RegisterPayload},
        services,
    },
    state::AppState,
};
use serde_json::Value;

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    services::login(&state, payload.username, payload.password).await
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    services::register(
        &state,
        payload.username,
        payload.password,
        payload.ik_pub,
        payload.spk_pub,
        payload.opk_pub,
    )
    .await
}

async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    services::refresh_token(&state, headers).await
}

pub fn auth_routes(app_state: AppState) -> Router<AppState> {
    let protected_by_refresh_routes = Router::new()
        .route("/refresh", get(refresh_token))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_refresh_token,
        ));
    let public = Router::new()
        .route("/register", post(register))
        .route("/login", post(login));

    Router::new()
        .nest("/auth", public)
        .nest("/auth", protected_by_refresh_routes)
}
