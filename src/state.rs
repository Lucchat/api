use crate::models::user::User;
use mongodb::Collection;
use shuttle_runtime::SecretStore;

#[derive(Clone)]
pub struct AppState {
    pub users: Collection<User>,
    pub secret_store: SecretStore,
}
