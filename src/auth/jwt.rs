use crate::auth::whitelist::is_jti_valid;
use crate::state::AppState;
use crate::utils::error::error_response;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::{http::Request, middleware::Next, response::Response};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shuttle_runtime::SecretStore;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub jti: String,
    pub exp: usize,
    pub token_type: String,
}

pub fn create_access_token(user_id: &str, secret_store: &SecretStore) -> String {
    create_jwt(user_id, secret_store, 15 * 60, "access")
}

pub fn create_refresh_token(user_id: &str, secret_store: &SecretStore) -> String {
    create_jwt(user_id, secret_store, 7 * 24 * 60 * 60, "refresh")
}

fn create_jwt(
    user_id: &str,
    secret_store: &SecretStore,
    expiration_secs: i64,
    token_type: &str,
) -> String {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expiration_secs))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        jti: Uuid::new_v4().to_string(),
        exp: expiration,
        token_type: token_type.to_string(),
    };

    let secret = secret_store.get("JWT_SECRET").expect("missing jwt_secret");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("JWT creation failed")
}

/// Décode un JWT reçu à l’aide du secret contenu dans le `SecretStore`.
pub fn decode_jwt(
    token: &str,
    secret_store: &SecretStore,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = secret_store
        .get("JWT_SECRET")
        .expect("JWT_SECRET not found in Secrets.toml");
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}

pub async fn require_access_token(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| error_response(StatusCode::UNAUTHORIZED, Some("Missing token")))?;

    let claims = decode_jwt(token, &state.secret_store)
        .map_err(|_| error_response(StatusCode::UNAUTHORIZED, None))?;

    if claims.token_type != "access" {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            Some("Expected access token"),
        ));
    }

    if !is_jti_valid(&state.redis, &claims.sub, &claims.jti, "access")
        .await
        .unwrap_or(false)
    {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            Some("Invalid token"),
        ));
    }

    req.extensions_mut().insert(claims.sub.clone());
    Ok(next.run(req).await)
}

pub async fn require_refresh_token(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| error_response(StatusCode::UNAUTHORIZED, Some("Missing token")))?;

    let claims = decode_jwt(token, &state.secret_store)
        .map_err(|_| error_response(StatusCode::UNAUTHORIZED, None))?;

    if !is_jti_valid(&state.redis, &claims.sub, &claims.jti, "refresh")
        .await
        .unwrap_or(false)
    {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            Some("Invalid token"),
        ));
    }

    if claims.token_type != "refresh" {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            Some("Expected refresh token"),
        ));
    }

    req.extensions_mut().insert(claims.sub.clone());
    Ok(next.run(req).await)
}
