use serde::{Deserialize, Serialize};

use crate::user::models::Opk;

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub ik_pub: [u8; 32],
    pub spk_pub: [u8; 32],
    pub opk_pub: Vec<Opk>,
}
