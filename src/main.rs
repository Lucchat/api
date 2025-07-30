use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use lucchat_api::{
    auth::jwt::{require_access_token, require_refresh_token},
    models::user::User,
    routes::{
        auth::{login, refresh_token, register},
        user::get_profile,
    },
    state::AppState,
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
        redis: redis,
    };

    let protected_by_refresh_routes = Router::new()
        .route("/auth/refresh", get(refresh_token))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_refresh_token,
        ));

    let protected_by_access_routes = Router::new()
        .route("/user/profile", get(get_profile))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_access_token,
        ));

    let app = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .merge(protected_by_refresh_routes)
        .merge(protected_by_access_routes)
        .with_state(app_state);

    Ok(app.into())
}
