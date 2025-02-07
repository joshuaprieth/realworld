use crate::{auth::Token, AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::headers;
use axum_extra::TypedHeader;
use headers::{Header, HeaderName, HeaderValue};
use serde::Serialize;
use std::{iter, sync::Arc};

pub async fn get_profile_authenticated(
    State(app): State<Arc<AppState>>,
    TypedHeader(token): TypedHeader<Token>,
    Path(username): Path<String>,
) {
}

#[derive(Debug, Serialize)]
pub struct Profile {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Debug, Serialize)]
pub struct ResponseProfile {
    profile: Profile,
}

pub async fn get_profile(
    State(app): State<Arc<AppState>>,
    TypedHeader(token): TypedHeader<Token>,
    Path(username): Path<String>,
) -> Json<ResponseProfile> {
    let profile = sqlx::query_as::<_, crate::database::Profile>(
        "
            SELECT `username`, `bio`, `image`, false AS `following`
            FROM `users`
            WHERE `users`.`username`=?
        ",
    )
    .bind(&username)
    .fetch_one(&app.db)
    .await
    .unwrap();

    Json(ResponseProfile { profile: Profile {
        username: profile.username,
        bio: profile.bio,
        image: profile.image,
        following: profile.following,
    } })
}
