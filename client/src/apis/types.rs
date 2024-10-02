use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct ListCategoriesResponse {
    pub result: Vec<String>,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct AttachmentInfo {
    pub id: usize,
    pub filename: String,
    pub url: String,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Note {
    pub id: usize,
    pub category: Option<String>,
    pub title: String,
    pub body: String,
    pub created: String,
    pub updated: String,
    pub attachments: Vec<AttachmentInfo>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ListNotesResponse {
    pub result: Vec<Note>,
}

pub type GetNoteResponse = Option<Note>;

#[derive(Serialize, Clone)]
pub struct CreateNoteRequest {
    pub title: String,
    pub body: String,
    pub category: Option<String>,
    pub attachments: Vec<usize>,
}

#[derive(Serialize, Clone)]
pub struct CreateAttachmentRequest {
    pub filename: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAttachmentResponse {
    pub id: usize,
    pub note_id: Option<usize>,
    pub filename: String,
    pub presigned_put_url: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct SearchRequest {
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub result: Vec<Note>,
}

#[derive(Serialize, Debug)]
pub struct ChatRequest {
    pub question: String,
    pub context: String,
}

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub response: String,
    pub note_id: usize,
}
