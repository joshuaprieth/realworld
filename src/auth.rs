use crate::AppState;
use axum::{extract::State, Json};
use axum_extra::headers;
use axum_extra::TypedHeader;
use headers::{Header, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{iter, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token(String);

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
    bio: Option<String>,
    image: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResponseUser {
    user: User,
}

#[derive(Debug, Deserialize)]
pub struct Authentication {
    user: AuthenticationUser,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticationUser {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct Registration {
    user: RegistrationUser,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationUser {
    username: String,
    email: String,
    password: String,
}

pub async fn authentication(
    State(state): State<Arc<AppState>>,
    Json(authenticate): Json<Authentication>,
) -> Json<ResponseUser> {
    println!("{:#?}", authenticate);

    Json(ResponseUser {
        user: User {
            email: String::from("jake@jake.jake"),
            token: String::from("jwt.token.here"),
            username: String::from("jake"),
            bio: Some(String::from("I work at statefarm")),
            image: None,
        },
    })
}

pub async fn registration(
    State(state): State<Arc<AppState>>,
    Json(registration): Json<Registration>,
) -> Json<ResponseUser> {
    sqlx::query(
        "
            INSERT INTO `users`
            (`email`, `password`, `username`)
            VALUES
            (?, ?, ?)
    ",
    )
    .bind(&registration.user.email)
    .bind(&registration.user.password)
    .bind(&registration.user.username)
    .execute(&state.db)
    .await
    .unwrap();

    #[derive(sqlx::FromRow, Debug)]
    struct UserA {
        email: String,
        password: String,
        username: String,
        bio: Option<String>,
        image: Option<String>,
    }
    let res = sqlx::query_as::<_, UserA>(
        "
            SELECT * from `users`
    ",
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    println!("{:#?}", res);

    Json(ResponseUser {
        user: User {
            email: registration.user.email.clone(),
            username: registration.user.username,
            token: registration.user.email,
            bio: None,
            image: None,
        },
    })
}

pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    TypedHeader(token): TypedHeader<Token>,
) -> Json<ResponseUser> {
    Json(ResponseUser {
        user: User {
            email: String::from("jake@jake.jake"),
            token: String::from("jwt.token.here"),
            username: String::from("jake"),
            bio: Some(String::from("I work at statefarm")),
            image: None,
        },
    })
}
