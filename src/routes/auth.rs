use crate::auth::{
    jwt::create_jwt,
    model::{LoginPayload, RegisterPayload},
    password::{hash_password, verify_password},
};
use crate::models::user::{User, UserPublic};
use crate::state::AppState;
use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Extension, Json};
use mongodb::bson::doc;
use serde_json::{json, Value};

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
        let user_public = UserPublic {
            uuid: user.uuid,
            username: user.username,
        };
        Ok(Json(json!({ "user": user_public, "token": token })))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<Value>, StatusCode> {
    let hashed = hash_password(&payload.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = User::new(payload.username.clone(), hashed);

    state.users.insert_one(&user).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = create_jwt(&user.uuid, &state.secret_store);
    Ok(Json(json!({
        "user": UserPublic {
            uuid: user.uuid,
            username: user.username,
        },
        "token": token
    })))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let old_token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    if let Some(token) = &old_token {
        println!("🔄 Ancien JWT : {}", token);
    } else {
        println!("⚠️ Aucun token trouvé dans l'en-tête Authorization");
    }

    let new_token = create_jwt(&user_id, &state.secret_store);
    Ok(Json(json!({ "token": new_token })))
}
