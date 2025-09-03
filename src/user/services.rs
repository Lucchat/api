use crate::{
    user::{
        models::{MessageInfo, User, UserPrivate, UserPublic, UserPublicFriend, UserResponse},
        payload::UserUpdatePayload,
        utils::{clean_reference, find_user, update_user_fields},
    },
    utils::error::error_response,
};
use axum::{http::StatusCode, Json};
use futures::stream::StreamExt;
use mongodb::{bson::doc, Collection};
use mongodb::bson::Document;
use serde_json::{json, Value};

pub async fn get_profile(
    users: Collection<User>,
    user_id: &str,
) -> Result<UserPrivate, (StatusCode, Json<Value>)> {
    let user = users
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
    users: Collection<User>,
    user_id: &str,
    target_id: &str,
) -> Result<Json<UserResponse>, (StatusCode, Json<Value>)> {
    let user = users
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

pub async fn get_all(users: Collection<User>) -> Result<Vec<UserPublic>, (StatusCode, Json<Value>)> {
    let mut cursor =
        users.find(doc! {}).await.map_err(|_| {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error"))
        })?;

    let mut users_public = Vec::new();
    while let Some(result) = cursor.next().await {
        if let Ok(user) = result {
            users_public.push(UserPublic {
                uuid: user.uuid,
                username: user.username,
                description: user.description,
                profile_picture: user.profile_picture,
            });
        }
    }

    Ok(users_public)
}

pub async fn update_user(
    users: Collection<User>,
    user_id: &str,
    updates: UserUpdatePayload,
) -> Result<UserPrivate, (StatusCode, Json<Value>)> {
    let mut set_doc: Document = Document::new();

    if let Some(username) = &updates.username {
        let existing_user = users
            .find_one(doc! { "username": username })
            .await
            .map_err(|_| {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error"))
            })?;

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

    users
        .update_one(doc! { "uuid": user_id }, update_doc)
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?;

    get_profile(users, user_id).await
}

pub async fn delete_user(users: Collection<User>, user_id: &str) -> Result<(), (StatusCode, Json<Value>)> {
    let user = find_user(&users, user_id).await?;

    let pending_requests = user.pending_friend_requests.clone();
    let friends_requests = user.friends_requests.clone();
    let friends = user.friends.clone();

    let result = users
        .delete_one(doc! { "uuid": user_id })
        .await
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, Some("Database error")))?;

    if result.deleted_count == 0 {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            Some("User not found"),
        ));
    }
    clean_reference(&users, friends, "friends", |u| &mut u.friends, user_id).await;

    clean_reference(
        &users,
        pending_requests,
        "friends_requests",
        |u| &mut u.friends_requests,
        user_id,
    )
    .await;

    clean_reference(
        &users,
        friends_requests,
        "pending_friend_requests",
        |u| &mut u.pending_friend_requests,
        user_id,
    )
    .await;

    Ok(())
}

pub async fn request_friendship(
    users: Collection<User>,
    user_id: &str,
    friend_id: &str,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<Value>)> {
    let mut user = find_user(&users, user_id).await?;

    let mut friend = find_user(&users, friend_id).await?;

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
        accept_friendship(users, user_id, friend_id).await?;
        return Ok(Json(
            json!({"message": "Friendship auto-accepted (mutual request)"}),
        ));
    }

    user.pending_friend_requests.push(friend_id.to_string());
    friend.friends_requests.push(user_id.to_string());

    update_user_fields(
        &users,
        user_id,
        doc! { "pending_friend_requests": user.pending_friend_requests },
    )
    .await?;
    update_user_fields(
        &users,
        friend_id,
        doc! { "friends_requests": friend.friends_requests },
    )
    .await?;

    Ok(Json(json!({"message": "Friend request sent!"})))
}

pub async fn accept_friendship(
    users: Collection<User>,
    user_id: &str,
    friend_id: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut user = find_user(&users, user_id).await?;
    let mut friend = find_user(&users, friend_id).await?;

    if user.uuid == friend.uuid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Cannot accept friend request from yourself"),
        ));
    }

    if !user.friends_requests.contains(&friend_id.to_string()) {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            Some("Friend request not found"),
        ));
    }
    if !friend
        .pending_friend_requests
        .contains(&user_id.to_string())
    {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            Some("Friend request not found on friend's side"),
        ));
    }

    if !friend
        .pending_friend_requests
        .contains(&user_id.to_string())
    {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            Some("Friend request not found on friend's side"),
        ));
    }

    user.friends_requests.retain(|id| id != friend_id);
    user.friends.push(friend_id.to_string());
    friend.pending_friend_requests.retain(|id| id != user_id);
    friend.friends.push(user_id.to_string());

    update_user_fields(
        &users,
        user_id,
        doc! { "friends_requests": user.friends_requests, "friends": user.friends },
    )
    .await?;
    update_user_fields(&users, friend_id, doc! { "pending_friend_requests": friend.pending_friend_requests, "friends": friend.friends }).await?;

    Ok(())
}

pub async fn decline_friendship(
    users: Collection<User>,
    user_id: &str,
    friend_id: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut user = find_user(&users, user_id).await?;
    let mut friend = find_user(&users, friend_id).await?;

    if user.uuid == friend.uuid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            Some("Cannot decline friendship with yourself"),
        ));
    }

    if !user.friends_requests.contains(&friend_id.to_string()) {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            Some("No pending friend request from this user"),
        ));
    }
    if !friend
        .pending_friend_requests
        .contains(&user_id.to_string())
    {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            Some("No pending friend request from this user"),
        ));
    }

    user.friends_requests.retain(|id| id != friend_id);
    friend.pending_friend_requests.retain(|id| id != user_id);

    update_user_fields(
        &users,
        user_id,
        doc! { "friends_requests": user.friends_requests },
    )
    .await?;
    update_user_fields(
        &users,
        friend_id,
        doc! { "pending_friend_requests": friend.pending_friend_requests },
    )
    .await?;

    Ok(())
}

pub async fn remove_friendship(
    users: Collection<User>,
    user_id: &str,
    friend_id: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut user = find_user(&users, user_id).await?;
    let mut friend = find_user(&users, friend_id).await?;

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

    update_user_fields(&users, user_id, doc! { "friends": user.friends }).await?;
    update_user_fields(&users, friend_id, doc! { "friends": friend.friends }).await?;

    Ok(())
}

pub async fn get_messages(
    users: Collection<User>,
    user_id: &str,
) -> Result<Vec<MessageInfo>, (StatusCode, Json<Value>)> {
    let user = find_user(&users, user_id).await?;
    let messages_info = user
        .unread_messages
        .iter()
        .map(|message| MessageInfo {
            uuid: message.uuid.clone(),
            sender: message.sender.clone(),
            receiver: message.receiver.clone(),
        })
        .collect();
    Ok(messages_info)
}
