use axum::{middleware, routing::{get, post}, Router};
use lucchat_api::{
    auth::jwt::require_jwt, models::user::User, routes::{auth::{login, refresh_token, register}, user::get_profile}, state::AppState
};
use mongodb::Collection;
use shuttle_runtime::SecretStore;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let mongo_uri = secret_store.get("MONGO_URI").expect("missing mongo_uri");
    let client = mongodb::Client::with_uri_str(&mongo_uri).await.unwrap();
    let db = client.database("lucchat");
    let users: Collection<User> = db.collection("users");

    let app_state = AppState {
        users,
        secret_store,
    };

    let protected_routes = Router::new()
        .route("/profile", get(get_profile))
        .route("/auth/refresh", get(refresh_token))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_jwt,
        ));

    let app = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .merge(protected_routes)
        .with_state(app_state);

    Ok(app.into())
}
