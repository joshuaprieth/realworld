mod auth;

use auth::{authentication, get_current_user};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/users/login", post(authentication))
        .route("/api/user", get(get_current_user))
        .with_state(Arc::new(AppState {}));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
struct AppState {}
