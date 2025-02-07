mod articles;
mod auth;
mod comments;
mod database;
mod profile;
mod token;

use articles::{
    create_article, delete_article, feed_articles, get_article, list_articles, update_article,
};
use auth::{authentication, get_current_user, registration, update_user};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use comments::{add_comment, get_comments};
use database::Pool;
use profile::{follow_user, get_profile, unfollow_user};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/users/login", post(authentication))
        .route("/api/users", post(registration))
        .route("/api/user", get(get_current_user))
        .route("/api/user", put(update_user))
        .route("/api/profiles/{username}", get(get_profile))
        .route("/api/profiles/{username}/follow", post(follow_user))
        .route("/api/profiles/{username}/follow", delete(unfollow_user))
        .route("/api/articles", get(list_articles))
        .route("/api/articles/feed", get(feed_articles))
        .route("/api/articles/{slug}", get(get_article))
        .route("/api/articles", post(create_article))
        .route("/api/articles/{slug}", put(update_article))
        .route("/api/articles/{slug}", delete(delete_article))
        .route("/api/articles/{slug}/comments", post(add_comment))
        .route("/api/articles/{slug}/comments", get(get_comments))
        .with_state(Arc::new(AppState {
            db: database::connect().await.unwrap(),
        }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
struct AppState {
    db: Pool,
}
