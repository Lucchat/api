use crate::auth::{
    jwt::{create_access_token, create_refresh_token, decode_jwt},
    model::{LoginPayload, RegisterPayload},
    password::{hash_password, verify_password},
    whitelist::set_valid_jti,
};
use crate::models::user::{User, UserPublic};
use crate::state::AppState;
use crate::utils::error::error_response;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use mongodb::bson::doc;
use serde_json::{json, Value};

use std::time::Instant;

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let start = Instant::now();

    let user = state
        .users
        .find_one(doc! { "username": &payload.username })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?
        .ok_or(error_response(
            StatusCode::UNAUTHORIZED,
            Some("Invalid credentials"),
        ))?;

    let after_find = start.elapsed();
    println!("🔍 find_one took: {after_find:?}", );

    let is_valid = verify_password(&payload.password, &user.password_hash).unwrap_or(false);

    let after_verify = start.elapsed();
    println!("🔐 password verify took: {:?}", after_verify - after_find);

    if is_valid {
        let access_token = create_access_token(&user.uuid, &state.secret_store);
        let refresh_token = create_refresh_token(&user.uuid, &state.secret_store);
        let access_claims = decode_jwt(&access_token, &state.secret_store).unwrap();
        let refresh_claims = decode_jwt(&refresh_token, &state.secret_store).unwrap();

        set_valid_jti(
            &state.redis,
            &access_claims.sub,
            &access_claims.jti,
            "access",
        )
        .await
        .unwrap();

        set_valid_jti(
            &state.redis,
            &refresh_claims.sub,
            &refresh_claims.jti,
            "refresh",
        )
        .await
        .unwrap();
        println!("✅ total login took: {:?}", start.elapsed());

        let user_public = UserPublic {
            uuid: user.uuid,
            username: user.username,
        };
        Ok(Json(json!({ "user": user_public, "token": {
            "access": access_token,
            "refresh": refresh_token
            } })))
    } else {
        println!("❌ login failed after {:?}", start.elapsed());
        Err(error_response(
            StatusCode::UNAUTHORIZED,
            Some("Invalid credentials"),
        ))
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Vérifie si l'utilisateur existe déjà
    let existing_user = state
        .users
        .find_one(doc! { "username": &payload.username })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?;

    if existing_user.is_some() {
        return Err(error_response(
            StatusCode::CONFLICT,
            Some("Username already taken"),
        ));
    }

    let hashed = hash_password(&payload.password)
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, None))?;
    let user = User::new(payload.username.clone(), hashed);

    state
        .users
        .insert_one(&user)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, None))?;

    let access_token = create_access_token(&user.uuid, &state.secret_store);
    let refresh_token = create_refresh_token(&user.uuid, &state.secret_store);
    let access_claims = decode_jwt(&access_token, &state.secret_store).unwrap();
    let refresh_claims = decode_jwt(&refresh_token, &state.secret_store).unwrap();
    set_valid_jti(
        &state.redis,
        &access_claims.sub,
        &access_claims.jti,
        "access",
    )
    .await
    .unwrap();

    set_valid_jti(
        &state.redis,
        &refresh_claims.sub,
        &refresh_claims.jti,
        "refresh",
    )
    .await
    .unwrap();

    Ok(Json(json!({
        "user": UserPublic {
            uuid: user.uuid,
            username: user.username,
        },
        "token": {
            "access": access_token,
            "refresh": refresh_token
            }
    })))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let old_token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(error_response(StatusCode::UNAUTHORIZED, None))?;

    // 1. Decode le token actuel
    let claims = decode_jwt(old_token, &state.secret_store)
        .map_err(|_| error_response(StatusCode::UNAUTHORIZED, None))?;

    if claims.token_type != "refresh" {
        return Err(error_response(StatusCode::UNAUTHORIZED, None));
    }

    // 4. Génère les nouveaux tokens
    let new_access = create_access_token(&claims.sub, &state.secret_store);
    let new_refresh = create_refresh_token(&claims.sub, &state.secret_store);
    let access_claims = decode_jwt(&new_access, &state.secret_store).unwrap();
    let refresh_claims = decode_jwt(&new_refresh, &state.secret_store).unwrap();
    set_valid_jti(
        &state.redis,
        &access_claims.sub,
        &access_claims.jti,
        "access",
    )
    .await
    .unwrap();

    set_valid_jti(
        &state.redis,
        &refresh_claims.sub,
        &refresh_claims.jti,
        "refresh",
    )
    .await
    .unwrap();

    Ok(Json(json!({
        "token": {
            "access": new_access,
            "refresh": new_refresh
        }
    })))
}
