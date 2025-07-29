use axum::{extract::{State, Extension}, Json};
use axum::http::StatusCode;
use crate::{models::user::UserPublic, state::AppState};
use mongodb::bson::doc;

pub async fn get_profile(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
) -> Result<Json<UserPublic>, StatusCode> {
    let user = state.users
        .find_one(doc! { "uuid": user_id })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserPublic {
        uuid: user.uuid,
        username: user.username,
    }))
}
