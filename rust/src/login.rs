use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {}

pub async fn login(
    State(db): State<SqlitePool>,
    Json(input): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}
