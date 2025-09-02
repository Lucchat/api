use crate::state::AppState;
use crate::user::utils::find_user;
use crate::utils::error::error_response;
use crate::{message::models::Message};
use axum::{http::StatusCode, Json};
use serde_json::Value;
use mongodb::bson::{doc, to_document};

pub async fn send_message(
    state: &AppState,
    user_id: &str,
    message: Message,
) -> Result<(), (StatusCode, Json<Value>)> {
    if user_id != message.sender {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("The message sender and the JWT id must be the same"),
        ));
    }

    if message.sender == message.receiver {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Sender and receiver cannot be the same"),
        ));
    }

    if let Err(_) = find_user(state, message.receiver.as_str()).await {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            Some("Receiver user does not exist"),
        ));
    }

    let msg_doc = to_document(&message).map_err(|e| {
           error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(&format!("Failed to convert message to document: {}", e)),
            )
        })?;
    state
        .users
        .update_one(
            doc! { "uuid": &message.receiver },
            doc! { "$push": { "unread_messages": msg_doc } },
        )
        .await
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(&format!("Failed to update unread messages: {}", e)),
            )
        })?;
    Ok(())
}

pub async fn read_message(
    state: &AppState,
    user_id: &str,
    message_id: &str,
) -> Result<Message, (StatusCode, Json<Value>)> {
    let update_result = state
        .users
        .find_one_and_update(
            doc! { "uuid": user_id, "unread_messages.uuid": message_id },
            doc! { "$pull": { "unread_messages": { "uuid": message_id } } },
        )
        .await
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(&format!("Failed to update user: {}", e)),
            )
        })?;

    let user = update_result.ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            Some("User or message not found"),
        )
    })?;

    let message = user
        .unread_messages
        .into_iter()
        .find(|msg| msg.uuid == message_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                Some("Message not found in unread messages"),
            )
        })?;

    Ok(message)
}
