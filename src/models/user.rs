use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub uuid: String,
    pub username: String,
    pub password_hash: String,
    pub keys: Key,
}

impl User {
    pub fn new(username: String, password_hash: String, keys: Key) -> Self {
        Self {
            id: None,
            uuid: Uuid::new_v4().to_string(),
            username,
            password_hash,
            keys,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPublic {
    pub uuid: String,
    pub username: String,
    pub keys: Key,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    pub ik_pub: [u8; 32],
    pub spk_pub: [u8; 32],
    pub opk_pub: Vec<[u8; 32]>
}

impl Key {
    pub fn new(ik_pub: [u8; 32], spk_pub: [u8; 32], opk_pub: Vec<[u8; 32]>) -> Self {
        Self {
            ik_pub,
            spk_pub,
            opk_pub,
        }
    }
}