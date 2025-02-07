use crate::token::{authenticate, create_token};
use crate::AppState;
use axum::body::Body;
use axum::extract::{FromRequestParts, OptionalFromRequestParts, State};
use axum::http::request::Parts;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AuthenticationFailure {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthenticationFailure {
    fn into_response(self) -> Response {
        Response::builder()
            .status(401)
            .body(Body::new(String::from("401 Unauthorized")))
            .unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Auth(pub i64);
impl<S> FromRequestParts<S> for Auth
where
    S: Sync + Send,
{
    type Rejection = AuthenticationFailure;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let mut values = parts.headers.get_all("authorization").iter();

        if let Some(value) = values.next() {
            if let Ok(string) = value.to_str() {
                if string.starts_with("Token ") {
                    let token = &string[6..];
                    let user_id = authenticate(token);

                    Ok(Auth(user_id))
                } else {
                    Err(AuthenticationFailure::InvalidToken)
                }
            } else {
                Err(AuthenticationFailure::InvalidToken)
            }
        } else {
            // There is no token header
            Err(AuthenticationFailure::MissingToken)
        }
    }
}

impl<S> OptionalFromRequestParts<S> for Auth
where
    S: Sync + Send,
{
    type Rejection = AuthenticationFailure;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <Auth as FromRequestParts<S>>::from_request_parts(parts, _state).await {
            Ok(auth) => Ok(Some(auth)),
            Err(AuthenticationFailure::MissingToken) => Ok(None),
            Err(AuthenticationFailure::InvalidToken) => Err(AuthenticationFailure::InvalidToken),
        }
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
            token: create_token(user.id),
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

    authentication(
        State(state),
        Json(Authentication {
            user: AuthenticationUser {
                email: registration.user.email,
                password: registration.user.password,
            },
        }),
    )
    .await
}

pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    Auth(user_id): Auth,
    headers: HeaderMap,
) -> Json<ResponseUser> {
    let user = sqlx::query_as::<_, crate::database::User>(
        "
            SELECT * FROM `users` WHERE `id`=?
    ",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .unwrap();

    Json(ResponseUser {
        user: User {
            email: user.email,
            token: headers
                .get("authorization")
                .unwrap() // Both this and the `to_str()`
                .to_str() // call will not panic because the `Auth`
                .unwrap() // extractor ensures that a header is present
                .to_owned(),
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
    Auth(user_id): Auth,
    headers: HeaderMap,
    Json(update): Json<Update>,
) -> Json<ResponseUser> {
    async fn update_field(state: &AppState, user_id: i64, name: &str, value: &Option<String>) {
        if let Some(value) = value {
            sqlx::query(&format!(
                "
                    UPDATE `users`
                    SET {}=?
                    WHERE `users`.`id`=?
                ",
                name
            ))
            .bind(&value)
            .bind(&user_id)
            .execute(&state.db)
            .await
            .unwrap();
        }
    }

    update_field(&state, user_id, "email", &update.user.email).await;
    update_field(&state, user_id, "password", &update.user.password).await;
    update_field(&state, user_id, "username", &update.user.username).await;
    update_field(&state, user_id, "bio", &update.user.bio).await;
    update_field(&state, user_id, "image", &update.user.image).await;

    get_current_user(State(state), Auth(user_id), headers).await
}
