use crate::auth::{
    jwt::create_jwt,
    model::{LoginPayload, RegisterPayload},
    password::{hash_password, verify_password},
};
use crate::models::user::{User, UserPublic};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mongodb::{bson::doc, Collection};
use serde_json::json;

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let user = state
        .users
        .find_one(doc! { "username": &payload.username })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if verify_password(&payload.password, &user.password_hash).unwrap_or(false) {
        let token = create_jwt(&user.uuid, &state.secret_store);
        Ok(Json(json!({ "token": token })))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<UserPublic>, (StatusCode, String)> {
    let hashed = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Hash error: {}", e),
        )
    })?;

    let user = User::new(payload.username.clone(), hashed);

    state.users.insert_one(&user).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )
    })?;

    Ok(Json(UserPublic {
        uuid: user.uuid,
        username: user.username,
    }))
}
