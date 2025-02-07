use crate::{auth::Auth, AppState};
use axum::{
    extract::{Path, State},
    Json,
};
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
    authentication: Option<Auth>,
    Path(username): Path<String>,
) -> Json<ResponseProfile> {
    let profile = if let Some(authentication) = authentication {
        let user_id = authentication.0;

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
    Auth(user_id): Auth,
    Path(username): Path<String>,
) -> Json<ResponseProfile> {
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

    get_profile(State(app), Some(Auth(user_id)), Path(username)).await
}

pub async fn unfollow_user(
    State(app): State<Arc<AppState>>,
    Auth(user_id): Auth,
    Path(username): Path<String>,
) -> Json<ResponseProfile> {
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

    get_profile(State(app), Some(Auth(user_id)), Path(username)).await
}
