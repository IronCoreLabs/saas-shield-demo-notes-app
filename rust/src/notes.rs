use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tracing::error;

use crate::db::{create_note, get_note};

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub org_id: u32,
    pub title: String,
    pub body: String,
    pub category: String,
    pub edek: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchNoteRequest {}

pub async fn get(
    Path(id): Path<u32>,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = get_note(&db, id).await.map_err(|e| {
        error!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(result))
}

pub async fn update(
    Path(id): Path<usize>,
    State(db): State<SqlitePool>,
    Json(input): Json<UpdateNoteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}

pub async fn create(
    State(db): State<SqlitePool>,
    Json(input): Json<CreateNoteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = create_note(&db, input).await.map_err(|e| {
        error!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(result))
}

pub async fn list(State(db): State<SqlitePool>) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}

pub async fn search(
    State(db): State<SqlitePool>,
    Json(input): Json<SearchNoteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(unimplemented!())
}
