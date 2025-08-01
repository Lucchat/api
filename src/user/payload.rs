use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdatePayload {
    pub username: Option<String>,
    pub description: Option<String>,
    pub profile_picture: Option<String>,
}
