use axum::{http::StatusCode, Json};
use redis::Client;
use serde_json::Value;
use shuttle_runtime::SecretStore;

use crate::auth::{
    jwt::{create_access_token, create_refresh_token, decode_jwt},
    whitelist::set_valid_jti,
};

pub async fn update_jwt(
    uuid: &str,
    secret_store: &SecretStore,
    redis: &Client,
) -> Result<(String, String), (StatusCode, Json<Value>)> {
    let access_token = create_access_token(uuid, secret_store);
    let refresh_token = create_refresh_token(uuid, secret_store);
    let access_claims = decode_jwt(&access_token, secret_store).unwrap();
    let refresh_claims = decode_jwt(&refresh_token, secret_store).unwrap();

    set_valid_jti(redis, &access_claims.sub, &access_claims.jti, "access")
        .await
        .unwrap();

    set_valid_jti(redis, &refresh_claims.sub, &refresh_claims.jti, "refresh")
        .await
        .unwrap();
    Ok((access_token, refresh_token))
}
