use serde::Serialize;
use sqlx::{prelude::FromRow, SqlitePool};

use crate::notes::CreateNoteRequest;

#[derive(Debug, FromRow, Serialize)]
pub struct NoteTable {
    pub id: u32,
    pub org_id: u32,
    pub title: String,
    pub body: String,
    pub category: String,
    pub edek: String,
    pub created: u64,
    pub updated: u64,
}

pub async fn create_sqlite_tables(pool: &SqlitePool) -> std::result::Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;

    sqlx::query(
        "CREATE TABLE organization (
      id          INTEGER PRIMARY KEY AUTOINCREMENT,
      login       TEXT NOT NULL,
      name        TEXT NOT NULL,
      password    TEXT NOT NULL,
      last_login  DATETIME DEFAULT current_timestamp,
      created     DATETIME DEFAULT current_timestamp,
      updated     DATETIME DEFAULT current_timestamp
      )",
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE note (
        id       INTEGER PRIMARY KEY AUTOINCREMENT,
        org_id   INTEGER NOT NULL,
        title    TEXT NOT NULL,
        body     TEXT NOT NULL,
        category TEXT NOT NULL,
        edek     TEXT NOT NULL,
        created  DATETIME DEFAULT current_timestamp,
        updated  DATETIME DEFAULT current_timestamp,
        FOREIGN KEY(org_id) REFERENCES organization(id)
        )",
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE attachment (
        id       INTEGER PRIMARY KEY AUTOINCREMENT,
        note_id  INTEGER NOT NULL,
        url      TEXT NOT NULL,
        FOREIGN KEY(note_id) REFERENCES note(id)
    )",
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn create_note(
    pool: &SqlitePool,
    note: CreateNoteRequest,
) -> std::result::Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query(
        "INSERT INTO note (org_id, title, body, category, edek) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(note.org_id)
    .bind(note.title)
    .bind(note.body)
    .bind(note.category)
    .bind(note.edek)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn get_note(
    pool: &SqlitePool,
    id: u32,
) -> std::result::Result<Option<NoteTable>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query_as::<_, NoteTable>("SELECT * FROM note WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await
}
