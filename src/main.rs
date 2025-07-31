use axum::Router;
use lucchat_api::{
    routes::{auth::auth_routes, user::user_routes},
    state::AppState,
    user::models::User,
};
use mongodb::Collection;
use shuttle_runtime::SecretStore;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let mongo_uri = secret_store.get("MONGO_URI").expect("missing mongo_uri");
    let redis_uri = secret_store.get("REDIS_URI").expect("missing REDIS_URI");
    let client = mongodb::Client::with_uri_str(&mongo_uri).await.unwrap();
    let db = client.database("lucchat");
    let users: Collection<User> = db.collection("users");
    let redis = redis::Client::open(redis_uri).expect("invalid redis URI");

    let app_state = AppState {
        users,
        secret_store,
        redis,
    };

    let user_routes = user_routes(app_state.clone());
    let auth_routes = auth_routes(app_state.clone());

    let app = Router::new()
        .merge(auth_routes)
        .merge(user_routes)
        .with_state(app_state);

    Ok(app.into())
}
