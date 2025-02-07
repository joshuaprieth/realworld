use crate::{auth::Token, token::authenticate, AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::headers;
use axum_extra::TypedHeader;
use serde::Serialize;
use std::sync::Arc;

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
    token: Option<TypedHeader<Token>>,
    Path(username): Path<String>,
) -> Json<ResponseProfile> {
    let profile = if let Some(token) = token {
        let user_id = authenticate(&token.0 .0);

        sqlx::query_as::<_, crate::database::Profile>(
            "
            SELECT `username`, `bio`, `image`, (
                SELECT COUNT(*)
                FROM `follows`
                WHERE `follows`.`source`=? AND `follows`.`target`=`users`.`id`
            ) AS `following`
            FROM `users`
            WHERE `users`.`username`=?
        ",
        )
        .bind(user_id)
        .bind(&username)
        .fetch_one(&app.db)
        .await
        .unwrap()
    } else {
        sqlx::query_as::<_, crate::database::Profile>(
            "
                SELECT `username`, `bio`, `image`, FALSE AS `following`
                FROM `users`
                WHERE `users`.`username`=?
            ",
        )
        .bind(&username)
        .fetch_one(&app.db)
        .await
        .unwrap()
    };

    Json(ResponseProfile {
        profile: Profile {
            username: profile.username,
            bio: profile.bio,
            image: profile.image,
            following: profile.following,
        },
    })
}

pub async fn follow_user(
    State(app): State<Arc<AppState>>,
    TypedHeader(token): TypedHeader<Token>,
    Path(username): Path<String>,
) -> Json<ResponseProfile> {
    let user_id = authenticate(&token.0);

    sqlx::query(
        "
            INSERT INTO `follows`
            (`source`, `target`)
            VALUES
            (?, (
                SELECT `id`
                FROM `users`
                WHERE `username`=?
            ))
        ",
    )
    .bind(user_id)
    .bind(&username)
    .execute(&app.db)
    .await
    .unwrap();

    get_profile(State(app), Some(TypedHeader(token)), Path(username)).await
}

pub async fn unfollow_user(
    State(app): State<Arc<AppState>>,
    TypedHeader(token): TypedHeader<Token>,
    Path(username): Path<String>,
) -> Json<ResponseProfile> {
    let user_id = authenticate(&token.0);

    sqlx::query(
        "
            DELETE FROM `follows`
            WHERE `source`=? AND `target`=(
                SELECT `id`
                FROM `users`
                WHERE `username`=?
            )
        ",
    )
    .bind(user_id)
    .bind(&username)
    .execute(&app.db)
    .await
    .unwrap();

    get_profile(State(app), Some(TypedHeader(token)), Path(username)).await
}
