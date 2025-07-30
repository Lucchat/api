use redis::{AsyncCommands, Client};

/// Met à jour le jti valide de l'utilisateur
pub async fn set_valid_jti(
    redis: &Client,
    user_id: &str,
    jti: &str,
    token_type: &str,
) -> redis::RedisResult<()> {
    if token_type != "access" && token_type != "refresh" {
        return Err(redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "Invalid token type",
        )));
    }
    let mut conn = redis.clone().get_multiplexed_async_connection().await?;
    conn.set(format!("{token_type}_jti:{user_id}"), jti)
        .await
}

/// Vérifie que le jti du token correspond à celui autorisé en Redis
pub async fn is_jti_valid(
    redis: &Client,
    user_id: &str,
    jti: &str,
    token_type: &str,
) -> redis::RedisResult<bool> {
    if token_type != "access" && token_type != "refresh" {
        return Err(redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "Invalid token type",
        )));
    }
    let mut conn = redis.clone().get_multiplexed_async_connection().await?;
    let expected: Option<String> = conn.get(format!("{token_type}_jti:{user_id}")).await?;
    Ok(expected.as_deref() == Some(jti))
}
