use crate::{auth::Auth, database::Profile, AppState};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite};
use std::iter;
use std::sync::Arc;

#[derive(Debug, FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct SimpleNoBodyArticle {
    id: i64,
    slug: String,
    title: String,
    description: String,
    created_at: String,
    updated_at: String,
    author: i64,
    favorited: bool,
    favorites_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoBodyArticle {
    slug: String,
    title: String,
    description: String,
    tag_list: Vec<String>,
    created_at: String,
    updated_at: String,
    favorited: bool,
    favorites_count: i64,
    author: Profile,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMultipleArticles {
    articles: Vec<NoBodyArticle>,
    articles_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct ListArticlesConstraints {
    tag: Option<String>,
    author: Option<String>,
    favorited: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn list_articles(
    State(app): State<Arc<AppState>>,
    authentication: Option<Auth>,
    Query(query): Query<ListArticlesConstraints>,
) -> Json<ResponseMultipleArticles> {
    if query.offset.is_some() && query.limit.is_none() {
        panic!("Offset must be used with limit");
    };

    let sql = format!(
        "
        SELECT `articles`.`id`, `slug`, `title`, `description`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `createdAt`) AS `createdAt`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `updatedAt`) AS `updatedAt`,
        `author`, (
            SELECT COUNT(*)
            FROM `favorites`
            WHERE `favorites`.`source`=? AND `favorites`.`target`=`articles`.`id`
        ) AS `favorited`, {}, (
            SELECT COUNT(*)
            FROM `favorites`
            WHERE `target`=`articles`.`id`
        ) AS `favoritesCount`
        FROM `articles`
        {}
        {}
        WHERE
        {}
        AND {}
        AND `favoritedByTarget`=TRUE
        ORDER BY `updatedAt` DESC
        {}
        {}
    ",
        query
            .favorited
            .as_ref()
            .map(|_| "(
            SELECT COUNT(*)
            FROM `favorites`
            WHERE `favorites`.`source`=? AND `favorites`.`target`=`articles`.`id`
        ) AS `favoritedByTarget`")
            .unwrap_or("TRUE AS `favoritedByTarget`"),
        query
            .tag
            .as_ref()
            .map(|_| "
                JOIN `taglist` ON `taglist`.`article`=`articles`.`id`
                JOIN `tags` ON `tags`.`id`=`taglist`.`tag`
            ")
            .unwrap_or(""),
        query
            .author
            .as_ref()
            .map(|_| "
                JOIN `users` ON `users`.`id`=`articles`.`author`
            ")
            .unwrap_or(""),
        query
            .tag
            .as_ref()
            .map(|_| "
                `tags`.`name`=?
            ")
            .unwrap_or("TRUE"),
        query
            .author
            .as_ref()
            .map(|_| "
                `users`.`username`=?
            ")
            .unwrap_or("TRUE"),
        query.limit.map(|_| "LIMIT ?").unwrap_or(""),
        query.offset.map(|_| "OFFSET ?").unwrap_or("")
    );

    // Get the list of article attributes first
    let statement = sqlx::query_as::<_, SimpleNoBodyArticle>(&sql)
        // Use an `id` which never exists if the user is not authenticated
        .bind(authentication.as_ref().map(|auth| auth.0).unwrap_or(-1));

    let statement = if let Some(name) = query.favorited {
        let id: i64 = sqlx::query_scalar(
            "
            SELECT `id`
            FROM `users`
            WHERE `username`=?
        ",
        )
        .bind(name)
        .fetch_one(&app.db)
        .await
        .unwrap();
        println!("has id {}", id);

        statement.bind(id)
    } else {
        statement
    };

    let statement = if let Some(name) = query.tag {
        statement.bind(name)
    } else {
        statement
    };

    let statement = if let Some(name) = query.author {
        statement.bind(name)
    } else {
        statement
    };

    let statement = if let Some(limit) = query.limit {
        statement.bind(limit)
    } else {
        statement
    };

    let statement = if let Some(offset) = query.offset {
        statement.bind(offset)
    } else {
        statement
    };

    let list = statement.fetch_all(&app.db).await.unwrap();

    // Then fetch the taglist
    let mut tag_list = Vec::with_capacity(list.len());

    for i in &list {
        let tags: Vec<String> = sqlx::query_scalar(
            "
                SELECT `name`
                FROM `taglist` INNER JOIN `tags` ON `taglist`.`tag`=`tags`.`id`
                WHERE `article`=?
            ",
        )
        .bind(i.id)
        .fetch_all(&app.db)
        .await
        .unwrap();
        tag_list.push(tags);
    }

    // Then the authors
    let mut author_list = Vec::with_capacity(list.len());

    for i in &list {
        let profile = if let Some(ref authentication) = authentication {
            let user_id = authentication.0;

            sqlx::query_as::<_, crate::database::Profile>(
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
            .bind(i.author)
        } else {
            sqlx::query_as::<_, crate::database::Profile>(
                "
                    SELECT `username`, `bio`, `image`, FALSE AS `following`
                    FROM `users`
                    WHERE `users`.`id`=?
                ",
            )
            .bind(i.author)
        }
        .fetch_one(&app.db)
        .await
        .unwrap();

        author_list.push(profile);
    }

    // Combine the query results
    let mut articles = Vec::with_capacity(list.len());

    for ((article, tag_list), author) in iter::zip(iter::zip(list, tag_list), author_list) {
        articles.push(NoBodyArticle {
            slug: article.slug,
            title: article.title,
            description: article.description,
            tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: article.favorited,
            favorites_count: article.favorites_count,
            author,
        });
    }

    Json(ResponseMultipleArticles {
        articles_count: articles.len(),
        articles,
    })
}

#[derive(Debug, Deserialize)]
pub struct FeedArticlesConstraints {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn feed_articles(
    State(app): State<Arc<AppState>>,
    Auth(user_id): Auth,
    Query(query): Query<FeedArticlesConstraints>,
) -> Json<ResponseMultipleArticles> {
    if query.offset.is_some() && query.limit.is_none() {
        panic!("Offset must be used with limit");
    };

    let sql = format!(
        "
        SELECT `articles`.`id`, `slug`, `title`, `description`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `createdAt`) AS `createdAt`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `updatedAt`) AS `updatedAt`,
        `author`, (
            SELECT COUNT(*)
            FROM `favorites`
            WHERE `favorites`.`source`=? AND `favorites`.`target`=`articles`.`id`
        ) AS `favorited`, (
            SELECT COUNT(*)
            FROM `favorites`
            WHERE `target`=`articles`.`id`
        ) AS `favoritesCount`
        FROM `articles`
        JOIN `follows` ON `follows`.`target`=`articles`.`author`
        WHERE `follows`.`source`=?
        ORDER BY `updatedAt` DESC
        {}
        {}
    ",
        query.limit.map(|_| "LIMIT ?").unwrap_or(""),
        query.offset.map(|_| "OFFSET ?").unwrap_or("")
    );

    // Get the list of article attributes first
    let statement = sqlx::query_as::<_, SimpleNoBodyArticle>(&sql)
        // Use an `id` which never exists if the user is not authenticated
        .bind(user_id)
        .bind(user_id);

    let statement = if let Some(limit) = query.limit {
        statement.bind(limit)
    } else {
        statement
    };

    let statement = if let Some(offset) = query.offset {
        statement.bind(offset)
    } else {
        statement
    };

    let list = statement.fetch_all(&app.db).await.unwrap();

    // Then fetch the taglist
    let mut tag_list = Vec::with_capacity(list.len());

    for i in &list {
        let tags: Vec<String> = sqlx::query_scalar(
            "
                SELECT `name`
                FROM `taglist` INNER JOIN `tags` ON `taglist`.`tag`=`tags`.`id`
                WHERE `article`=?
            ",
        )
        .bind(i.id)
        .fetch_all(&app.db)
        .await
        .unwrap();
        tag_list.push(tags);
    }

    // Then the authors
    let mut author_list = Vec::with_capacity(list.len());

    for i in &list {
        let profile = sqlx::query_as::<_, crate::database::Profile>(
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
        .bind(i.author)
        .fetch_one(&app.db)
        .await
        .unwrap();

        author_list.push(profile);
    }

    // Combine the query results
    let mut articles = Vec::with_capacity(list.len());

    for ((article, tag_list), author) in iter::zip(iter::zip(list, tag_list), author_list) {
        articles.push(NoBodyArticle {
            slug: article.slug,
            title: article.title,
            description: article.description,
            tag_list,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: article.favorited,
            favorites_count: article.favorites_count,
            author,
        });
    }

    Json(ResponseMultipleArticles {
        articles_count: articles.len(),
        articles,
    })
}

#[derive(Debug, FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct SimpleBodyArticle {
    id: i64,
    slug: String,
    title: String,
    description: String,
    body: String,
    created_at: String,
    updated_at: String,
    author: i64,
    favorited: bool,
    favorites_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyArticle {
    slug: String,
    title: String,
    description: String,
    body: String,
    tag_list: Vec<String>,
    created_at: String,
    updated_at: String,
    favorited: bool,
    favorites_count: i64,
    author: Profile,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseSingleArticle {
    article: BodyArticle,
}

pub async fn get_article(
    State(app): State<Arc<AppState>>,
    authentication: Option<Auth>,
    Path(slug): Path<String>,
) -> Json<ResponseSingleArticle> {
    let article = sqlx::query_as::<_, SimpleBodyArticle>(
        "
        SELECT `id`, `slug`, `title`, `description`, `body`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `createdAt`) AS `createdAt`,
            strftime('%Y-%m-%dT%H:%M:%fZ', `updatedAt`) AS `updatedAt`,
            `author`, (
                SELECT COUNT(*)
                FROM `favorites`
                WHERE `favorites`.`source`=? AND `favorites`.`target`=`articles`.`id`
            ) AS `favorited`, (
                SELECT COUNT(*)
                FROM `favorites`
                WHERE `target`=`articles`.`id`
            ) AS `favoritesCount`
        FROM `articles`
        WHERE `slug`=?
        ",
    )
    .bind(authentication.as_ref().map(|auth| auth.0).unwrap_or(-1))
    .bind(slug)
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Fetch the taglist
    let tags: Vec<String> = sqlx::query_scalar(
        "
            SELECT `name`
            FROM `taglist` INNER JOIN `tags` ON `taglist`.`tag`=`tags`.`id`
            WHERE `article`=?
        ",
    )
    .bind(article.id)
    .fetch_all(&app.db)
    .await
    .unwrap();

    // Then the authors
    let profile = if let Some(ref authentication) = authentication {
        let user_id = authentication.0;

        sqlx::query_as::<_, crate::database::Profile>(
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
        .bind(article.author)
    } else {
        sqlx::query_as::<_, crate::database::Profile>(
            "
                    SELECT `username`, `bio`, `image`, FALSE AS `following`
                    FROM `users`
                    WHERE `users`.`id`=?
                ",
        )
        .bind(article.author)
    }
    .fetch_one(&app.db)
    .await
    .unwrap();

    Json(ResponseSingleArticle {
        article: BodyArticle {
            slug: article.slug,
            title: article.title,
            description: article.description,
            body: article.body,
            tag_list: tags,
            created_at: article.created_at,
            updated_at: article.updated_at,
            favorited: article.favorited,
            favorites_count: article.favorites_count,
            author: profile,
        },
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticleRequest {
    article: CreateArticle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticle {
    title: String,
    description: String,
    body: String,
    tag_list: Option<Vec<String>>,
}

pub async fn create_article(
    State(app): State<Arc<AppState>>,
    Auth(user_id): Auth,
    Json(article): Json<CreateArticleRequest>,
) -> Json<ResponseSingleArticle> {
    let article = article.article;
    let slug = create_slug(&article.title);

    let id: i64 = sqlx::query_scalar(
        "
            INSERT INTO `articles`
            (`slug`, `title`, `description`, `body`, `author`)
            VALUES
            (?, ?, ?, ?, ?)
            RETURNING `id`
        ",
    )
    .bind(&slug)
    .bind(article.title)
    .bind(article.description)
    .bind(article.body)
    .bind(user_id)
    .fetch_one(&app.db)
    .await
    .unwrap();

    if let Some(mut tag_list) = article.tag_list {
        if tag_list.len() > 999 {
            panic!("Too many tags");
        };

        if !tag_list.is_empty() {
            // We only need each tag once
            tag_list.sort();
            tag_list.dedup();

            let tags = sqlx::query_as::<_, (String, i64)>(
                "
                SELECT `name`, `id`
                FROM `tags`
            ",
            )
            .fetch_all(&app.db)
            .await
            .unwrap();

            let mut query: QueryBuilder<Sqlite> =
                QueryBuilder::new("INSERT INTO `taglist` (`article`, `tag`) ");

            query.push_values(tag_list, |mut query, tag| {
                query
                    .push_bind(id)
                    .push_bind(
                        tags.iter()
                            .find_map(|(name, id)| if name == &tag { Some(id) } else { None }),
                    );
            });

            query.build().execute(&app.db).await.unwrap();
        };
    };

    get_article(State(app), Some(Auth(user_id)), Path(slug)).await
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleRequest {
    article: UpdateArticle,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticle {
    title: Option<String>,
    description: Option<String>,
    body: Option<String>,
}

pub async fn update_article(
    State(app): State<Arc<AppState>>,
    Auth(user_id): Auth,
    Path(slug): Path<String>,
    Json(article): Json<UpdateArticleRequest>,
) -> Json<ResponseSingleArticle> {
    let article = article.article;

    let article_id: i64 = sqlx::query_scalar(
        "
            SELECT `id`
            FROM `articles`
            WHERE `slug`=? AND `author`=?
        ",
    )
    .bind(&slug)
    .bind(user_id) // Make sure only the owner can edit an article
    .fetch_one(&app.db)
    .await
    .unwrap();

    if article.title.is_none() && article.description.is_none() && article.body.is_none() {
        return get_article(State(app), Some(Auth(user_id)), Path(slug)).await;
    };

    let attributes = vec![
        article.title.as_ref().map(|_| "`slug`=?"), // Update the slug if the title changes
        article.title.as_ref().map(|_| "`title`=?"),
        article.description.as_ref().map(|_| "`description`=?"),
        article.body.as_ref().map(|_| "`body`=?"),
    ];

    let new_slug = article
        .title
        .as_ref()
        .map(|slug| create_slug(&slug))
        .unwrap_or_else(|| slug);

    let sql = format!(
        "
            UPDATE `articles`
            SET
            {}, `updatedAt`=CURRENT_TIMESTAMP
            WHERE `id`=?
        ",
        attributes
            .into_iter()
            .filter_map(|i| i)
            .collect::<Vec<&'static str>>()
            .join(", ")
    );

    let query = sqlx::query(&sql);

    let query = if let Some(_) = article.title {
        query.bind(&new_slug)
    } else {
        query
    };

    let query = if let Some(ref title) = article.title {
        query.bind(title)
    } else {
        query
    };

    let query = if let Some(description) = article.description {
        query.bind(description)
    } else {
        query
    };

    let query = if let Some(body) = article.body {
        query.bind(body)
    } else {
        query
    };

    query.bind(article_id).execute(&app.db).await.unwrap();

    get_article(State(app), Some(Auth(user_id)), Path(new_slug)).await
}

pub async fn delete_article(
    State(app): State<Arc<AppState>>,
    Auth(user_id): Auth,
    Path(slug): Path<String>,
) {
    let article_id: i64 = sqlx::query_scalar(
        "
            SELECT `id`
            FROM `articles`
            WHERE `slug`=? AND `author`=?
        ",
    )
    .bind(slug)
    .bind(user_id) // Make sure only owners can delete their articles
    .fetch_one(&app.db)
    .await
    .unwrap();

    // First, delete the favorites
    sqlx::query(
        "
            DELETE FROM `favorites`
            WHERE `target`=?
        ",
    )
    .bind(article_id)
    .execute(&app.db)
    .await
    .unwrap();

    // Then, delete the tags
    sqlx::query(
        "
        DELETE FROM `taglist`
        WHERE `article`=?
    ",
    )
    .bind(article_id)
    .execute(&app.db)
    .await
    .unwrap();

    // Finally, delete the articles
    sqlx::query(
        "
            DELETE FROM `articles`
            WHERE `id`=?
        ",
    )
    .bind(article_id)
    .execute(&app.db)
    .await
    .unwrap();
}

pub async fn favorite_article(
    State(app): State<Arc<AppState>>,
    Auth(user_id): Auth,
    Path(slug): Path<String>,
)  -> Json<ResponseSingleArticle> {
    sqlx::query(
        "
            INSERT INTO `favorites`
            (`source`, `target`)
            SELECT ?, `id`
            FROM `articles`
            WHERE `articles`.`slug`=?
        ",
    )
    .bind(user_id)
    .bind(&slug)
    .execute(&app.db)
    .await
    .unwrap();

    get_article(State(app), Some(Auth(user_id)), Path(slug)).await
}

fn create_slug(string: &str) -> String {
    string.replace(' ', "-").to_lowercase()
}
