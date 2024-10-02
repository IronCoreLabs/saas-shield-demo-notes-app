use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Serialize;
use tracing::error;

use crate::{
    db::{self},
    AppState, CurrentOrganization,
};

#[derive(Debug, Serialize)]
pub struct CategoryListResponse {
    result: Vec<String>,
}

pub async fn list(
    State(AppState { db, sdk, .. }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = db::list_categories(&db, org, sdk).await.map_err(|e| {
        error!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(CategoryListResponse { result }))
}
