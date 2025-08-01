use crate::{state::AppState, user::models::User, utils::error::error_response};
use axum::{http::StatusCode, Json};
use mongodb::bson::{doc, Document};
use serde_json::Value;

pub async fn find_user(state: &AppState, uuid: &str) -> Result<User, (StatusCode, Json<Value>)> {
    state
        .users
        .find_one(doc! { "uuid": uuid })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?
        .ok_or(error_response(
            StatusCode::NOT_FOUND,
            Some("User not found"),
        ))
}

pub async fn update_user_fields(
    state: &AppState,
    uuid: &str,
    updates: Document,
) -> Result<(), (StatusCode, Json<Value>)> {
    state
        .users
        .update_one(doc! { "uuid": uuid }, doc! { "$set": updates })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, None))?;

    Ok(())
}

pub async fn clean_reference(
    state: &AppState,
    user_ids: Vec<String>,
    field_name: &str,
    field_extractor: impl Fn(&mut User) -> &mut Vec<String>,
    user_id: &str,
) {
    let tasks = user_ids.into_iter().map(|fid| {
        let field_extractor = &field_extractor;
        async move {
            if let Ok(mut friend) = find_user(state, &fid).await {
                field_extractor(&mut friend).retain(|id| id != user_id);
                let _ = update_user_fields(state, &fid, doc! { field_name: field_extractor(&mut friend).clone() }).await;
            }
        }
    });
    futures::future::join_all(tasks).await;
}
