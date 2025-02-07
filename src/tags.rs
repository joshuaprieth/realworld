use crate::AppState;
use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct ResponseTagList {
    tags: Vec<String>,
}

pub async fn get_tags(State(app): State<Arc<AppState>>) -> Json<ResponseTagList> {
    Json(ResponseTagList {
        tags: sqlx::query_scalar(
            "
            SELECT `name`
            FROM `tags`
        ",
        )
        .fetch_all(&app.db)
        .await
        .unwrap(),
    })
}
