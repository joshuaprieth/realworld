use sqlx::sqlite::SqlitePoolOptions;

pub use sqlx::sqlite::SqlitePool as Pool;

pub async fn connect() -> Result<Pool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    sqlx::query(
        "
            CREATE TABLE `users` (
                email VARCHAR(32) NOT NULL UNIQUE,
                password VARCHAR(64) NOT NULL,
                username VARCHAR(32) NOT NULL UNIQUE,
                bio VARCHAR(256) NULL,
                image VARCHAR(256) NULL
            )
        ",
    )
    .execute(&pool).await?;

    Ok(pool)
}
