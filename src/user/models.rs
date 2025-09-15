use crate::message::models::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub uuid: String,
    pub username: String,
    pub password_hash: String,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
    pub pending_friend_requests: Vec<String>,
    pub friends_requests: Vec<String>,
    pub friends: Vec<String>,
    pub keys: Key,
    pub unread_messages: Vec<Message>,
}

impl User {
    pub fn new(username: String, password_hash: String, keys: Key) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            username,
            password_hash,
            description: None,
            profile_picture: None,
            keys,
            pending_friend_requests: Vec::new(),
            friends_requests: Vec::new(),
            friends: Vec::new(),
            unread_messages: Vec::new(),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserResponse {
    Public(UserPublic),
    PublicFriend(UserPublicFriend),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPublic {
    pub uuid: String,
    pub username: String,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPublicFriend {
    pub uuid: String,
    pub username: String,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
    pub keys: Key,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPrivate {
    pub uuid: String,
    pub username: String,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
    pub keys: Key,
    pub pending_friend_requests: Vec<String>,
    pub friends_requests: Vec<String>,
    pub friends: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageInfo {
    pub uuid: String,
    pub sender: String,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    pub ik_pub: [u8; 32],
    pub spk_pub: [u8; 32],
    pub opk_pub: Vec<Opk>,
}

impl Key {
    pub fn new(ik_pub: [u8; 32], spk_pub: [u8; 32], opk_pub: Vec<Opk>) -> Self {
        Self {
            ik_pub,
            spk_pub,
            opk_pub,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Opk {
    pub uuid: String,
    pub key: [u8; 32],
}