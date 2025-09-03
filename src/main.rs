use axum::Router;
use lucchat_api::{
    routes::{auth::auth_routes, message::message_routes, user::user_routes},
    state::AppState,
};
use shuttle_runtime::SecretStore;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let mongo_uri = secret_store.get("MONGO_URI").expect("missing mongo_uri");
    let redis_uri = secret_store.get("REDIS_URI").expect("missing REDIS_URI");
    let mongo = mongodb::Client::with_uri_str(&mongo_uri).await.unwrap();
    let redis = redis::Client::open(redis_uri).expect("invalid redis URI");

    let app_state = AppState {
        mongo,
        secret_store,
        redis,
        started_at: std::time::Instant::now(),
    };

    let user_routes = user_routes(app_state.clone());
    let auth_routes = auth_routes(app_state.clone());
    let message_routes = message_routes(app_state.clone());

    let app = Router::new()
        .merge(auth_routes)
        .merge(user_routes)
        .merge(message_routes)
        .with_state(app_state);

    Ok(app.into())
}
