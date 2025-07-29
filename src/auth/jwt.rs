use axum::extract::State;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shuttle_runtime::SecretStore;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

/// Génère un JWT pour l'utilisateur à partir du store de secrets Shuttle.
pub fn create_jwt(user_id: &str, secret_store: &SecretStore) -> String {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
    };

    let secret = secret_store
        .get("JWT_SECRET")
        .expect("JWT_SECRET not found in Secrets.toml");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("JWT encoding failed")
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

use axum::{http::Request, middleware::Next, response::Response};
use axum::http::StatusCode;
use crate::state::AppState;

pub async fn require_jwt(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let Some(token) = auth_header.and_then(|s| s.strip_prefix("Bearer ")) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let claims = match decode_jwt(token, &state.secret_store) {
        Ok(c) => c,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    req.extensions_mut().insert(claims.sub.clone());
    Ok(next.run(req).await)
}
