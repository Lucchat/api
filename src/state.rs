use mongodb::Collection;
use shuttle_runtime::SecretStore;

use crate::user::models::User;

#[derive(Clone)]
pub struct AppState {
    pub mongo: mongodb::Client,
    pub secret_store: SecretStore,
    pub redis: redis::Client,
    pub started_at: std::time::Instant,
}

impl AppState {
    pub fn get_user_collection(&self) -> Collection<User> {
        self.mongo.database("lucchat").collection("users")
    }
}