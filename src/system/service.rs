use axum::{http::StatusCode, Json};
use mongodb::bson::doc;
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn get_health(state: &AppState) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let uptime = state.started_at.elapsed().as_secs();
    let mongo_status = match state
        .mongo
        .database("admin")
        .run_command(doc! {"ping": 1})
        .await
    {
        Ok(_) => "up",
        Err(_) => "down",
    };

    let redis_status = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(mut conn) => match redis::cmd("PING").query_async::<String>(&mut conn).await {
            Ok(_) => "up",
            Err(_) => "down",
        },
        Err(_) => "down",
    };

    let dependencies = json!({
        "mongo": mongo_status,
        "redis": redis_status
    });

    let timestamp = chrono::Utc::now().timestamp();

    Ok(Json(json!({
        "status": "ok",
        "uptime": uptime,
        "dependencies": dependencies,
        "timestamp": timestamp,
    })))
}

pub async fn get_version(_: &crate::state::AppState) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Ok(Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "git_hash": option_env!("GIT_HASH").unwrap_or("unknown"),
        "build_time": option_env!("BUILD_TIME").unwrap_or("unknown"),
    })))
}


