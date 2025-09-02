use crate::{
    auth::jwt::require_access_token,
    message::{
        models::{self, Message},
        services,
    },
    state::AppState,
};
use axum::{
    extract::Path,
    middleware,
    routing::{get, post},
    Router,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

async fn send_message(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(message): Json<models::Message>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    services::send_message(&state, &user_id, message).await?;
    Ok(Json(json!({"status": "Message sent successfully"})))
}

async fn read_message(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Path(message_id): Path<String>,
) -> Result<Json<Message>, (StatusCode, Json<Value>)> {
    let message = services::read_message(&state, &user_id, &message_id).await?;
    Ok(Json(message))
}

pub fn message_routes(app_state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/send", post(send_message))
        .route("/read/{message_id}", get(read_message))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_access_token,
        ));

    Router::new().nest("/message", protected)
}
