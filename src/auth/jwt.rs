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
    println!("Using secret: {}", secret); // Debugging line to check the secret
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
