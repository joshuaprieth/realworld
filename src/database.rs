use sqlx::{sqlite::SqlitePoolOptions, FromRow};

pub use sqlx::sqlite::SqlitePool as Pool;

pub async fn connect() -> Result<Pool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:realworld.db")
        .await?;

    Ok(pool)
}

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Profile {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

pub async fn current_user(pool: &Pool, token: &str) -> User {
    sqlx::query_as::<_, User>(
        "
            SELECT `id`, `email`, `password`, `username`, `bio`, `image`
            FROM `users`
            WHERE `users`.`email`=?
        ",
    )
    .bind(token)
    .fetch_one(pool)
    .await
    .unwrap()
}
