use sqlx::sqlite::SqlitePoolOptions;

pub use sqlx::sqlite::SqlitePool as Pool;

pub async fn connect() -> Result<Pool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
}
