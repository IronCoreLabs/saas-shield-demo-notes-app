use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    db::{self},
    AppState, CurrentOrganization,
};

#[derive(Debug, Deserialize)]
pub struct CreateAttachmentRequest {
    pub filename: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAttachmentResponse {
    pub id: u32,
    pub note_id: Option<u32>,
    pub filename: String,
    pub presigned_put_url: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentInfo {
    pub id: u32,
    pub filename: String,
    pub url: String,
}

pub async fn create(
    State(AppState { db, aws_sdk, .. }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
    Json(input): Json<CreateAttachmentRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = db::create_attachment(&db, input, &org, aws_sdk)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(result))
}
