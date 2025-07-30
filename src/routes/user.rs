use crate::{models::user::UserPublic, state::AppState, utils::error::error_response};
use axum::http::StatusCode;
use axum::{
    extract::{Extension, State},
    Json,
};
use mongodb::bson::doc;
use serde_json::Value;

pub async fn get_profile(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<Json<UserPublic>, (StatusCode, Json<Value>)> {
    let user = state
        .users
        .find_one(doc! { "uuid": user_id })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?
        .ok_or(error_response(
            StatusCode::NOT_FOUND,
            Some("User not found"),
        ))?;

    Ok(Json(UserPublic {
        uuid: user.uuid,
        username: user.username,
    }))
}
