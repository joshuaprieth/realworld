use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_extra::headers;
use axum_extra::TypedHeader;
use headers::{Header, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter, sync::Arc};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Token(String);

static TOKEN: HeaderName = HeaderName::from_static("authorization");

impl Header for Token {
    fn name() -> &'static HeaderName {
        &TOKEN
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let first = values.next().ok_or_else(headers::Error::invalid)?;
        let string = first.to_str().map_err(|_| headers::Error::invalid())?;

        if string.starts_with("Token ") {
            Ok(Token(string[6..].to_string()))
        } else {
            Err(headers::Error::invalid())
        }
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(iter::once(
            HeaderValue::from_str(&format!("Token {}", self.0)).unwrap(),
        ));
    }
}

#[derive(Debug, Clone, Serialize)]
struct User {
    email: String,
    token: String,
    username: String,
    bio: String,
    image: Option<String>,
}

#[derive(Debug, Serialize)]
struct ResponseUser {
    user: User,
}

#[derive(Debug, Deserialize)]
struct Authentication {
    user: AuthenticationUser,
}

#[derive(Debug, Deserialize)]
struct AuthenticationUser {
    email: String,
    password: String,
}

async fn authentication(
    State(state): State<Arc<AppState>>,
    Json(authenticate): Json<Authentication>,
) -> Json<ResponseUser> {
    println!("{:#?}", authenticate);

    Json(ResponseUser {
        user: User {
            email: String::from("jake@jake.jake"),
            token: String::from("jwt.token.here"),
            username: String::from("jake"),
            bio: String::from("I work at statefarm"),
            image: None,
        },
    })
}

async fn get_current_user(
    State(state): State<Arc<AppState>>,
    TypedHeader(token): TypedHeader<Token>,
) -> Json<ResponseUser> {
    Json(ResponseUser {
        user: User {
            email: String::from("jake@jake.jake"),
            token: String::from("jwt.token.here"),
            username: String::from("jake"),
            bio: String::from("I work at statefarm"),
            image: None,
        },
    })
}
