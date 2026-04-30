use axum::Router;
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use lucchat_api::{
    routes::{
        auth::auth_routes, message::message_routes, system::system_routes, user::user_routes,
    },
    state::{AppState, SecretStore},
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mongo_uri = std::env::var("MONGO_URI").expect("missing MONGO_URI");
    let redis_uri = std::env::var("REDIS_URI").expect("missing REDIS_URI");
    let mongo = mongodb::Client::with_uri_str(&mongo_uri).await.unwrap();
    let redis = redis::Client::open(redis_uri).expect("invalid redis URI");
    let secret_store = SecretStore;

    let app_state = AppState {
        mongo,
        secret_store,
        redis,
        started_at: std::time::Instant::now(),
    };

    let user_routes = user_routes(app_state.clone());
    let auth_routes = auth_routes(app_state.clone());
    let message_routes = message_routes(app_state.clone());
    let system_routes = system_routes();

    let app= Router::new()
        .merge(auth_routes)
        .merge(user_routes)
        .merge(message_routes)
        .merge(system_routes)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();

}
