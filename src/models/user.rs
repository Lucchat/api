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
}

impl User {
    pub fn new(username: String, password_hash: String) -> Self {
        Self {
            id: None,
            uuid: Uuid::new_v4().to_string(),
            username,
            password_hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPublic {
    pub uuid: String,
    pub username: String,
}
