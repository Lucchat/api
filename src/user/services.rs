use crate::{
    state::AppState,
    user::{
        models::{UserPrivate, UserPublic, UserPublicFriend, UserResponse},
        payload::UserUpdatePayload,
        utils::{find_user, update_user_fields},
    },
    utils::error::error_response,
};
use axum::{http::StatusCode, Json};
use futures::stream::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::Document;
use serde_json::{json, Value};

pub async fn get_profile(
    state: &AppState,
    user_id: &str,
) -> Result<UserPrivate, (StatusCode, Json<Value>)> {
    let user = state
        .users
        .find_one(doc! { "uuid": user_id })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?
        .ok_or(error_response(
            StatusCode::NOT_FOUND,
            Some("User not found"),
        ))?;

    Ok(UserPrivate {
        uuid: user.uuid,
        username: user.username,
        description: user.description,
        profile_picture: user.profile_picture,
        keys: user.keys,
        pending_friend_requests: user.pending_friend_requests,
        friends_requests: user.friends_requests,
        friends: user.friends,
    })
}

pub async fn get_by_id(
    state: &AppState,
    user_id: &str,
    target_id: &str,
) -> Result<Json<UserResponse>, (StatusCode, Json<Value>)> {
    let user = state
        .users
        .find_one(doc! { "uuid": target_id })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?
        .ok_or(error_response(
            StatusCode::NOT_FOUND,
            Some("User not found"),
        ))?;

    let is_friend = user.friends.contains(&user_id.to_string());

    if is_friend {
        let user_friend = UserPublicFriend {
            uuid: user.uuid,
            username: user.username,
            keys: user.keys,
            description: user.description,
            profile_picture: user.profile_picture,
        };
        Ok(Json(UserResponse::PublicFriend(user_friend)))
    } else {
        let user_public = UserPublic {
            uuid: user.uuid,
            username: user.username,
            description: user.description,
            profile_picture: user.profile_picture,
        };
        Ok(Json(UserResponse::Public(user_public)))
    }
}

pub async fn get_all(state: &AppState) -> Result<Vec<UserPublic>, (StatusCode, Json<Value>)> {
    let mut cursor =
        state.users.find(doc! {}).await.map_err(|_| {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error"))
        })?;

    let mut users = Vec::new();
    while let Some(result) = cursor.next().await {
        if let Ok(user) = result {
            users.push(UserPublic {
                uuid: user.uuid,
                username: user.username,
                description: user.description,
                profile_picture: user.profile_picture,
            });
        }
    }

    Ok(users)
}

pub async fn update_user(
    state: &AppState,
    user_id: &str,
    updates: UserUpdatePayload,
) -> Result<UserPrivate, (StatusCode, Json<Value>)> {
    let mut set_doc = Document::new();

    // Vérifier si le nouveau username existe déjà (et qu'il n'est pas le sien)
    if let Some(username) = &updates.username {
        let existing_user = state
            .users
            .find_one(doc! { "username": username })
            .await
            .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?;

        if let Some(user) = existing_user {
            if user.uuid != user_id {
                return Err(error_response(
                    StatusCode::BAD_REQUEST,
                    Some("Username already taken"),
                ));
            }
        }
    }

    
    if let Some(username) = &updates.username {
        set_doc.insert("username", username);
    }
    if let Some(description) = &updates.description {
        set_doc.insert("description", description);
    }
    if let Some(profile_picture) = &updates.profile_picture {
        set_doc.insert("profile_picture", profile_picture);
    }

    if set_doc.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("No fields to update"),
        ));
    }

    let update_doc = doc! { "$set": set_doc };

    state
        .users
        .update_one(doc! { "uuid": user_id }, update_doc)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?;

    Ok(get_profile(state, user_id).await?)
}

pub async fn request_friendship(
    state: &AppState,
    user_id: &str,
    friend_id: &str,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<Value>)> {
    let mut user = find_user(state, user_id).await?;

    let mut friend = find_user(state, friend_id).await?;

    if user.uuid == friend.uuid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Cannot send friend request to yourself"),
        ));
    }

    if user.friends.contains(&friend_id.to_string()) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Already friends"),
        ));
    }

    if friend.friends.contains(&user_id.to_string()) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Already friends"),
        ));
    }

    if user
        .pending_friend_requests
        .contains(&friend_id.to_string())
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Friend request already sent"),
        ));
    }

    if friend.friends_requests.contains(&user_id.to_string()) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Friend request already sent"),
        ));
    }

    if user.friends_requests.contains(&friend_id.to_string())
        && friend
            .pending_friend_requests
            .contains(&user_id.to_string())
    {
        accept_friendship(state, user_id, friend_id).await?;
        return Ok(Json(
            json!({"message": "Friendship auto-accepted (mutual request)"}),
        ));
    }

    user.pending_friend_requests.push(friend_id.to_string());
    friend.friends_requests.push(user_id.to_string());

    update_user_fields(
        state,
        user_id,
        doc! { "pending_friend_requests": user.pending_friend_requests },
    )
    .await?;
    update_user_fields(
        state,
        friend_id,
        doc! { "friends_requests": friend.friends_requests },
    )
    .await?;

    Ok(Json(json!({"message": "Friend request sent!"})))
}

pub async fn accept_friendship(
    state: &AppState,
    user_id: &str,
    friend_id: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut user = find_user(state, user_id).await?;
    let mut friend = find_user(state, friend_id).await?;

    if user.uuid == friend.uuid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Cannot accept friend request from yourself"),
        ));
    }

    if !user.friends_requests.contains(&friend_id.to_string()) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Friend request not found"),
        ));
    }
    if !friend
        .pending_friend_requests
        .contains(&user_id.to_string())
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Friend request not found on friend's side"),
        ));
    }

    if !friend
        .pending_friend_requests
        .contains(&user_id.to_string())
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Friend request not found on friend's side"),
        ));
    }

    user.friends_requests.retain(|id| id != friend_id);
    user.friends.push(friend_id.to_string());
    friend.pending_friend_requests.retain(|id| id != user_id);
    friend.friends.push(user_id.to_string());

    update_user_fields(
        state,
        user_id,
        doc! { "friends_requests": user.friends_requests, "friends": user.friends },
    )
    .await?;
    update_user_fields(state, friend_id, doc! { "pending_friend_requests": friend.pending_friend_requests, "friends": friend.friends }).await?;

    Ok(())
}

pub async fn decline_friendship(
    state: &AppState,
    user_id: &str,
    friend_id: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut user = find_user(state, user_id).await?;
    let mut friend = find_user(state, friend_id).await?;

    if user.uuid == friend.uuid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Cannot decline friendship with yourself"),
        ));
    }

    if !user.friends_requests.contains(&friend_id.to_string()) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("No pending friend request from this user"),
        ));
    }
    if !friend
        .pending_friend_requests
        .contains(&user_id.to_string())
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("No pending friend request from this user"),
        ));
    }

    user.friends_requests.retain(|id| id != friend_id);
    friend.pending_friend_requests.retain(|id| id != user_id);

    update_user_fields(
        state,
        user_id,
        doc! { "friends_requests": user.friends_requests },
    )
    .await?;
    update_user_fields(
        state,
        friend_id,
        doc! { "pending_friend_requests": friend.pending_friend_requests },
    )
    .await?;

    Ok(())
}

pub async fn remove_friendship(
    state: &AppState,
    user_id: &str,
    friend_id: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut user = find_user(state, user_id).await?;
    let mut friend = find_user(state, friend_id).await?;

    if user.uuid == friend.uuid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Cannot remove friendship with yourself"),
        ));
    }

    if !user.friends.contains(&friend_id.to_string()) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Not friends with this user"),
        ));
    }
    if !friend.friends.contains(&user_id.to_string()) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Not friends with this user"),
        ));
    }

    user.friends.retain(|id| id != friend_id);
    friend.friends.retain(|id| id != user_id);

    update_user_fields(state, user_id, doc! { "friends": user.friends }).await?;
    update_user_fields(state, friend_id, doc! { "friends": friend.friends }).await?;

    Ok(())
}
