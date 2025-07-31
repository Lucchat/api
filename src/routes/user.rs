use crate::{
    auth::jwt::require_access_token,
    state::AppState,
    user::{
        models::{UserPrivate, UserPublic, UserResponse},
        services,
    },
};
use axum::{
    extract::Path,
    middleware,
    routing::{delete, get, post},
    Router,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

async fn get_profile(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<Json<UserPrivate>, (StatusCode, Json<Value>)> {
    let user = services::get_profile(&state, &user_id).await?;
    Ok(Json(user))
}

async fn get_by_id(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Path(id): Path<String>,
) -> Result<Json<UserResponse>, (StatusCode, Json<Value>)> {
    let user = services::get_by_id(&state, &user_id, &id).await?;
    Ok(user)
}

async fn get_all(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserPublic>>, (StatusCode, Json<Value>)> {
    let users = services::get_all(&state).await?;
    Ok(Json(users))
}

async fn request_friendship(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(user_id): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = services::request_friendship(&state, &user_id, &username).await;
    match result {
        Ok(value) => Ok(value),
        Err(err) => Err(err),
    }
}

async fn accept_friendship(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(user_id): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = services::accept_friendship(&state, &user_id, &username).await;
    match result {
        Ok(_) => Ok(Json(json!({"message": "Friend request accepted"}))),
        Err(err) => Err(err),
    }
}

async fn reject_friendship(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(user_id): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = services::decline_friendship(&state, &user_id, &username).await;
    match result {
        Ok(_) => Ok(Json(json!({"message": "Friend request rejected"}))),
        Err(err) => Err(err),
    }
}

async fn remove_friendship(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(user_id): Extension<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = services::remove_friendship(&state, &user_id, &username).await;
    match result {
        Ok(_) => Ok(Json(json!({"message": "Friend removed"}))),
        Err(err) => Err(err),
    }
}

pub fn user_routes(app_state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/me", get(get_profile))
        .route("/", get(get_all))
        .route("/:id", get(get_by_id))
        .route("/:id/friends/requests", post(request_friendship))
        .route("/:id/friends/requests/accept", post(accept_friendship))
        .route("/:id/friends/requests/reject", post(reject_friendship))
        .route("/:id/friends", delete(remove_friendship))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_access_token,
        ));

    Router::new().nest("/user", protected)
}
