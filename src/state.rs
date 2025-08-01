use crate::user::models::User;
use mongodb::Collection;
use shuttle_runtime::SecretStore;

#[derive(Clone)]
pub struct AppState {
    pub users: Collection<User>,
    pub secret_store: SecretStore,
    pub redis: redis::Client,
}
