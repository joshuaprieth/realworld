use crate::{auth::Auth, database::Profile, AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;

#[derive(Debug, FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct Comment {
    id: i64,
    created_at: String,
    updated_at: String,
    body: String,
    author: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseComment {
    id: i64,
    created_at: String,
    updated_at: String,
    body: String,
    author: Profile,
}

#[derive(Serialize)]
pub struct ResponseSingleComment {
    comment: ResponseComment,
}

#[derive(Deserialize)]
pub struct AddComment {
    body: String,
}

#[derive(Deserialize)]
pub struct RequestAddComment {
    comment: AddComment,
}

pub async fn add_comment(
    State(app): State<Arc<AppState>>,
    Auth(user_id): Auth,
    Path(slug): Path<String>,
    Json(comment): Json<RequestAddComment>,
) -> Json<ResponseSingleComment> {
    let comment = comment.comment;

    let article_id: i64 = sqlx::query_scalar(
        "
            SELECT `id`
            FROM `articles`
            WHERE `slug`=?
        ",
    )
    .bind(slug)
    .fetch_one(&app.db)
    .await
    .unwrap();

    let comment = sqlx::query_as::<_, Comment>(
        "
            INSERT INTO `comments` (`article`, `body`, `author`)
            VALUES (?, ?, ?)
            RETURNING `id`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `createdAt`) AS `createdAt`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `updatedAt`) AS `updatedAt`,
            `body`, `author`
        ",
    )
    .bind(article_id)
    .bind(comment.body)
    .bind(user_id)
    .fetch_one(&app.db)
    .await
    .unwrap();

    let author = sqlx::query_as::<_, crate::database::Profile>(
        "
        SELECT `username`, `bio`, `image`, (
            SELECT COUNT(*)
            FROM `follows`
            WHERE `follows`.`source`=? AND `follows`.`target`=`users`.`id`
        ) AS `following`
        FROM `users`
        WHERE `users`.`id`=?
    ",
    )
    .bind(user_id)
    .bind(comment.author)
    .fetch_one(&app.db)
    .await
    .unwrap();

    Json(ResponseSingleComment {
        comment: ResponseComment {
            id: comment.id,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            body: comment.body,
            author,
        },
    })
}

#[derive(Debug, FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct CommentWithAuthor {
    id: i64,
    created_at: String,
    updated_at: String,
    body: String,
    username: String,
    bio: Option<String>,
    image: Option<String>,
    following: bool,
}

#[derive(Serialize)]
pub struct ResponseMultipleComments {
    comments: Vec<ResponseComment>,
}

pub async fn get_comments(
    State(app): State<Arc<AppState>>,
    authentication: Option<Auth>,
    Path(slug): Path<String>,
) -> Json<ResponseMultipleComments> {
    let article_id: i64 = sqlx::query_scalar(
        "
            SELECT `id`
            FROM `articles`
            WHERE `slug`=?
        ",
    )
    .bind(slug)
    .fetch_one(&app.db)
    .await
    .unwrap();

    let comments = sqlx::query_as::<_, CommentWithAuthor>(
        "
            SELECT `comments`.`id`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `createdAt`) AS `createdAt`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `updatedAt`) AS `updatedAt`,
            `body`, `username`, `bio`, `image`, (
                SELECT COUNT(*)
                FROM `follows`
                WHERE `follows`.`source`=? AND `follows`.`target`=`users`.`id`
            ) AS `following`
            FROM `comments`
            JOIN `users` ON `users`.`id`=`comments`.`author`
            WHERE `article`=?
        ",
    )
    .bind(authentication.as_ref().map(|auth| auth.0).unwrap_or(-1))
    .bind(article_id)
    .fetch_all(&app.db)
    .await
    .unwrap();

    Json(ResponseMultipleComments {
        comments: comments
            .into_iter()
            .map(|comment| ResponseComment {
                id: comment.id,
                created_at: comment.created_at,
                updated_at: comment.updated_at,
                body: comment.body,
                author: Profile {
                    username: comment.username,
                    bio: comment.bio,
                    image: comment.image,
                    following: comment.following,
                },
            })
            .collect(),
    })
}
