use crate::user::models::MessageInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub uuid: String,
    pub sender: String,
    pub receiver: String,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub ratchet_pub: [u8; 32], // DH public key used in ratchet step
    pub message_index: u32,    // Index in chain key (CKs.index)
    pub opk_used: Option<[u8; 32]>,
    pub ek_used: Option<[u8; 32]>,
    pub created_at: i64,
}

impl Message {
    pub fn message_info(&self) -> MessageInfo {
        MessageInfo {
            uuid: self.uuid.clone(),
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}
