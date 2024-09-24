use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct NotesUpdateRequest {}

#[derive(Debug, Deserialize)]
pub struct NotesCreateRequest {}
#[derive(Debug, Deserialize)]
pub struct NotesSearchRequest {}

pub async fn get(
    Path(id): Path<usize>,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}

pub async fn update(
    Path(id): Path<usize>,
    State(db): State<SqlitePool>,
    Json(input): Json<NotesUpdateRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}

pub async fn create(
    State(db): State<SqlitePool>,
    Json(input): Json<NotesCreateRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}

pub async fn list(State(db): State<SqlitePool>) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}

pub async fn search(
    State(db): State<SqlitePool>,
    Json(input): Json<NotesSearchRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}
