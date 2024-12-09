use anyhow::anyhow;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use ironcore_alloy::{
    standard::{EdekWithKeyIdHeader, StandardDocumentOps},
    AlloyMetadata, DocumentId, TenantId,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    db::{self, EncryptedString, Note},
    embeddings::{self, generate_query_embeddings},
    search_service::{self, QueryType},
    AppState, CurrentOrganization,
};

#[derive(Clone, Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub body: String,
    pub category: Option<String>,
    #[serde(default)]
    pub attachments: Vec<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListQuery {
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NoteListResponse {
    result: Vec<Note>,
}

#[derive(Debug, Serialize)]
pub struct NoteSearchResponse {
    result: Vec<Note>,
}

pub type UpdateNoteRequest = CreateNoteRequest;

#[derive(Clone, Debug, Deserialize)]
pub struct SearchNoteRequest {
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QueryChatbotRequest {
    pub question: String,
}

impl From<&QueryChatbotRequest> for SearchNoteRequest {
    fn from(value: &QueryChatbotRequest) -> Self {
        SearchNoteRequest {
            title: Some(value.question.clone()),
            body: Some(value.question.clone()),
        }
    }
}

fn handle_err(e: anyhow::Error) -> StatusCode {
    error!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn get(
    Path(id): Path<u32>,
    State(AppState {
        db, sdk, aws_sdk, ..
    }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = db::get_note(&db, id, &org, sdk, aws_sdk)
        .await
        .map_err(handle_err)?;

    Ok(Json(result))
}

pub async fn update(
    Path(id): Path<u32>,
    State(AppState {
        db,
        sdk,
        es_sdk,
        aws_sdk,
        ai_sdk,
    }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
    Json(input): Json<UpdateNoteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let db_result = db::update_note(&db, input.clone(), id, &org, sdk.clone(), aws_sdk)
        .await
        .map_err(handle_err)?;
    let embeddings = embeddings::generate_and_encrypt_embedding(ai_sdk, sdk, input.clone(), &org)
        .await
        .map_err(handle_err)?;
    search_service::update_note(id, input, &org, es_sdk, embeddings)
        .await
        .map_err(handle_err)?;
    Ok(Json(db_result))
}

pub async fn rekey(
    Path(id): Path<u32>,
    State(AppState { db, sdk, .. }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
) -> Result<impl IntoResponse, StatusCode> {
    let edek = db::get_edek(&db, id, &org)
        .await
        .and_then(|maybe_note| {
            maybe_note.ok_or(anyhow!(format!(
                "Note with id {} not found for user {}",
                id, org.0.login
            )))
        })
        .map_err(handle_err)?;

    let tenant_id = TenantId(org.0.login.clone());
    let metadata = AlloyMetadata::new_simple(tenant_id);
    let document_id = DocumentId("".to_string());
    let edeks = vec![(
        document_id.clone(),
        EdekWithKeyIdHeader(edek.edek.to_enc_bytes().map_err(handle_err)?),
    )]
    .into_iter()
    .collect();
    let result = sdk
        .standard()
        .rekey_edeks(edeks, &metadata, None)
        .await
        .map_err(|e| handle_err(e.into()))?;

    db::put_edek(
        &db,
        id,
        &org,
        EncryptedString::new(result.successes.get(&document_id).unwrap().0.clone()),
    )
    .await
    .map_err(handle_err)?;

    Ok(Json(()))
}
pub async fn create(
    State(AppState {
        db,
        sdk,
        es_sdk,
        aws_sdk,
        ai_sdk,
    }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
    Json(input): Json<CreateNoteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let db_result = db::create_note(&db, input.clone(), &org, sdk.clone(), aws_sdk)
        .await
        .map_err(handle_err)?;
    let embeddings = embeddings::generate_and_encrypt_embedding(ai_sdk, sdk, input.clone(), &org)
        .await
        .map_err(handle_err)?;
    search_service::index_note(db_result.id, input, &org, es_sdk, embeddings)
        .await
        .map_err(handle_err)?;

    Ok(Json(db_result))
}

pub async fn list(
    State(AppState {
        db, sdk, aws_sdk, ..
    }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
    query: Query<ListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = db::list_notes(&db, org, sdk, query.category.clone(), aws_sdk)
        .await
        .map(|notes| NoteListResponse { result: notes })
        .map_err(handle_err)?;

    Ok(Json(result))
}

pub async fn search(
    State(AppState {
        db,
        sdk,
        es_sdk,
        aws_sdk,
        ..
    }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
    Json(input): Json<SearchNoteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let found_ids = search_service::query_notes(
        &org,
        es_sdk,
        QueryType::Keyword {
            title: input.title,
            body: input.body,
        },
    )
    .await
    .map_err(handle_err)?;
    let result = db::search_notes(&db, found_ids, &org, sdk, aws_sdk)
        .await
        .map(|notes| NoteSearchResponse { result: notes })
        .map_err(handle_err)?;

    Ok(Json(result))
}

pub async fn chat(
    State(AppState {
        db,
        sdk,
        es_sdk,
        aws_sdk,
        ai_sdk,
    }): State<AppState>,
    Extension(org): Extension<CurrentOrganization>,
    Json(input): Json<QueryChatbotRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let embeddings = generate_query_embeddings(ai_sdk.clone(), sdk.clone(), (&input).into(), &org)
        .await
        .map_err(handle_err)?;
    // TODO: an improvement here would be to take in ids as part of the request that are already known to be part of
    //       the context of the conversation, and pull them out to add to the context, so the conversation could
    //       continue while referencing earlier notes
    let found_ids = search_service::query_notes(&org, es_sdk, QueryType::Knn { embeddings })
        .await
        .map_err(handle_err)?;
    match found_ids.first() {
        Some(first_id) => {
            let decrypted_note = db::search_notes(&db, vec![*first_id], &org, sdk, aws_sdk)
                .await
                .map(|notes| NoteSearchResponse { result: notes })
                .map_err(handle_err)?;
            let result = embeddings::query_chatbot(
                ai_sdk,
                decrypted_note
                    .result
                    .first()
                    .cloned()
                    .unwrap_or_else(Note::default),
                input,
            )
            .await
            .map_err(handle_err)?;

            Ok(Json(result))
        }
        None => Err(handle_err(anyhow!("Vector search returned no results."))),
    }
}
