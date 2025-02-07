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

pub async fn authentication(
    State(state): State<Arc<AppState>>,
    Json(authenticate): Json<Authentication>,
) -> Json<ResponseUser> {
    let user = sqlx::query_as::<_, crate::database::User>(
        "
            SELECT * FROM `users` WHERE `email`=? AND `password`=?
    ",
    )
    .bind(&authenticate.user.email)
    .bind(&authenticate.user.password)
    .fetch_one(&state.db)
    .await
    .unwrap();

    Json(ResponseUser {
        user: User {
            email: user.email.clone(),
            token: user.email,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    })
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
    let user = sqlx::query_as::<_, crate::database::User>(
        "
            SELECT * FROM `users` WHERE `email`=?
    ",
    )
    .bind(token.0)
    .fetch_one(&state.db)
    .await
    .unwrap();

    Json(ResponseUser {
        user: User {
            email: user.email.clone(),
            token: user.email,
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    })
}

#[derive(Debug, Deserialize)]
pub struct Update {
    user: UpdateUser,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    email: Option<String>,
    password: Option<String>,
    username: Option<String>,
    bio: Option<String>,
    image: Option<String>,
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    TypedHeader(token): TypedHeader<Token>,
    Json(update): Json<Update>,
) -> Json<ResponseUser> {
    async fn update_field(state: &AppState, token: &Token, name: &str, value: &Option<String>) {
        if let Some(value) = value {
            sqlx::query(&format!(
                "
                    UPDATE `users`
                    SET {}=?
                    WHERE `users`.`email`=?
                ",
                name
            ))
            .bind(&value)
            .bind(&token.0)
            .execute(&state.db)
            .await
            .unwrap();
        }
    }

    update_field(&state, &token, "email", &update.user.email).await;
    update_field(&state, &token, "password", &update.user.password).await;
    update_field(&state, &token, "username", &update.user.username).await;
    update_field(&state, &token, "bio", &update.user.bio).await;
    update_field(&state, &token, "image", &update.user.image).await;

    get_current_user(State(state), TypedHeader(token)).await
}
