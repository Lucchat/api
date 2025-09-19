use crate::auth::password::is_password_strong;
use crate::auth::utils::update_jwt;
use crate::state::AppState;
use crate::user::models::{OneTimePreKeyPublic, User, UserPrivate};
use crate::utils::error::error_response;
use crate::{
    auth::{
        jwt::decode_jwt,
        password::{hash_password, verify_password},
    },
    user::models::Key,
};
use axum::{
    http::{HeaderMap, StatusCode},
    Json,
};
use mongodb::bson::doc;
use mongodb::Collection;
use serde_json::{json, Value};
use shuttle_runtime::SecretStore;

pub async fn login(
    users: Collection<User>,
    secret_store: SecretStore,
    redis_client: redis::Client,
    username: String,
    password: String,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user = users
        .find_one(doc! { "username": username })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?
        .ok_or(error_response(
            StatusCode::UNAUTHORIZED,
            Some("Invalid credentials"),
        ))?;

    let is_valid = verify_password(&password, &user.password_hash).unwrap_or(false);

    if is_valid {
        let (access_token, refresh_token) =
            update_jwt(&user.uuid, &secret_store, &redis_client).await?;

        let user_private = UserPrivate {
            uuid: user.uuid,
            username: user.username,
            keys: user.keys,
            description: user.description,
            profile_picture: user.profile_picture,
            pending_friend_requests: user.pending_friend_requests,
            friends_requests: user.friends_requests,
            friends: user.friends,
        };
        Ok(Json(json!({ 
            "user": user_private, 
            "token": {
                "access": access_token,
                "refresh": refresh_token
            } })))
    } else {
        Err(error_response(
            StatusCode::UNAUTHORIZED,
            Some("Invalid credentials"),
        ))
    }
}

pub async fn register(
    users: Collection<User>,
    secret_store: SecretStore,
    redis_client: redis::Client,
    username: String,
    password: String,
    ik_pub: [u8; 32],
    spk_pub: [u8; 32],
    opk_pub: Vec<OneTimePreKeyPublic>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let existing_user = users
        .find_one(doc! { "username": username.clone() })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?;

    if existing_user.is_some() {
        return Err(error_response(
            StatusCode::CONFLICT,
            Some("Username already taken"),
        ));
    }

    is_password_strong(&password)?;

    let hashed = hash_password(&password)
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, None))?;
    let keys = Key::new(ik_pub, spk_pub, opk_pub);
    let user = User::new(username, hashed, keys);

    users
        .insert_one(&user)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, None))?;

    let (access_token, refresh_token) =
        update_jwt(&user.uuid, &secret_store, &redis_client).await?;

    let user_private = UserPrivate {
        uuid: user.uuid,
        username: user.username,
        keys: user.keys,
        description: user.description,
        profile_picture: user.profile_picture,
        pending_friend_requests: user.pending_friend_requests,
        friends_requests: user.friends_requests,
        friends: user.friends,
    };
    Ok(Json(json!({
        "user": user_private,
        "token": {
            "access": access_token,
            "refresh": refresh_token
        }
    })))
}

pub async fn refresh_token(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let old_token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(error_response(StatusCode::UNAUTHORIZED, None))?;

    let claims = decode_jwt(old_token, &state.secret_store)
        .map_err(|_| error_response(StatusCode::UNAUTHORIZED, None))?;

    if claims.token_type != "refresh" {
        return Err(error_response(StatusCode::UNAUTHORIZED, None));
    }

    let (new_access, new_refresh) =
        update_jwt(&claims.sub, &state.secret_store, &state.redis).await?;
    Ok(Json(json!({
        "token": {
            "access": new_access,
            "refresh": new_refresh
        }
    })))
}
