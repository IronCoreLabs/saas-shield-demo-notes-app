pub mod types;

use crate::components::chatbot::{ChatMessage, Sender};
use anyhow::Result;
use lazy_static::lazy_static;
use reqwasm::http::Request;
use std::env::var;
use types::{
    ChatRequest, ChatResponse, CreateAttachmentRequest, CreateAttachmentResponse,
    CreateNoteRequest, GetNoteResponse, ListCategoriesResponse, ListNotesResponse, Note,
    SearchRequest, SearchResponse,
};

lazy_static! {
    pub static ref SERVER_BASE_URL: String =
        var("SERVER_BASE_URL").unwrap_or_else(|_| "http://localhost:7654/".to_string());
}

pub static API_SUBPATH: &str = "api/";
pub static CATEGORIES_API: &str = "categories/";
pub static NOTES_API: &str = "notes/";
pub static SEARCH_API: &str = "search/";
pub static CHAT_API: &str = "chat/";
pub static ATTACHMENTS_API: &str = "attachments/";

pub async fn categories() -> Result<ListCategoriesResponse> {
    let url = format!("{}{API_SUBPATH}{CATEGORIES_API}", *SERVER_BASE_URL);
    let categories = Request::get(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await?
        .json::<ListCategoriesResponse>()
        .await?;

    Ok(categories)
}

pub async fn notes(category_filter: Option<String>) -> Result<ListNotesResponse> {
    let mut url = format!("{}{API_SUBPATH}{NOTES_API}", *SERVER_BASE_URL);
    if let Some(cat_filter) = category_filter {
        url.push_str(&format!("?category={cat_filter}"));
    };
    let notes = Request::get(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await?
        .json::<ListNotesResponse>()
        .await?;

    Ok(notes)
}

pub async fn create_note(
    title: String,
    body: String,
    category: Option<String>,
    attachments: Vec<usize>,
) -> Result<Note> {
    let url = format!("{}{API_SUBPATH}{NOTES_API}", *SERVER_BASE_URL);
    let note = Request::post(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .body(serde_json::to_string(&CreateNoteRequest {
            title,
            body,
            category,
            attachments,
        })?)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<Note>()
        .await?;

    Ok(note)
}

pub async fn rekey_note(note_id: usize) -> Result<()> {
    let url = format!(
        "{}{API_SUBPATH}{NOTES_API}{note_id}/rekey",
        *SERVER_BASE_URL
    );
    Request::put(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    Ok(())
}

pub async fn note(note_id: usize) -> Result<GetNoteResponse> {
    let url = format!("{}{API_SUBPATH}{NOTES_API}{note_id}", *SERVER_BASE_URL);
    let note = Request::get(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await?
        .json::<GetNoteResponse>()
        .await?;

    Ok(note)
}

pub async fn update_note(
    note_id: usize,
    title: String,
    body: String,
    category: Option<String>,
    attachments: Vec<usize>,
) -> Result<Note> {
    let url = format!("{}{API_SUBPATH}{NOTES_API}{note_id}", *SERVER_BASE_URL);
    let note = Request::put(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .body(serde_json::to_string(&CreateNoteRequest {
            title,
            body,
            category,
            attachments,
        })?)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<Note>()
        .await?;

    Ok(note)
}

pub async fn search(query: String) -> Result<SearchResponse> {
    let url = format!("{}{API_SUBPATH}{NOTES_API}{SEARCH_API}", *SERVER_BASE_URL);
    let notes = Request::post(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .body(serde_json::to_string(&SearchRequest {
            title: Some(query.clone()),
            body: Some(query),
        })?)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<SearchResponse>()
        .await?;

    Ok(notes)
}

pub async fn create_attachment(filename: String) -> Result<CreateAttachmentResponse> {
    let url = format!("{}{API_SUBPATH}{ATTACHMENTS_API}", *SERVER_BASE_URL);
    Ok(Request::post(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .body(serde_json::to_string(&CreateAttachmentRequest {
            filename,
        })?)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<CreateAttachmentResponse>()
        .await?)
}

pub async fn write_to_url(url: String, data: Vec<u8>) -> Result<()> {
    Request::put(&url)
        .header("Content-Type", "application/octet-stream")
        .body(<&[u8] as Into<Box<[u8]>>>::into(&data[..]))
        .send()
        .await?;

    Ok(())
}

pub async fn query_chatbot(mut chat_messages: Vec<ChatMessage>) -> Result<ChatResponse> {
    let url = format!("{}{API_SUBPATH}{CHAT_API}", *SERVER_BASE_URL);
    let question = chat_messages.remove(chat_messages.len() - 1);
    let notes = Request::post(&url)
        .credentials(web_sys::RequestCredentials::Include)
        .body(serde_json::to_string(&ChatRequest {
            question: format!("[INST]{}[/INST]", question.message),
            context: chat_messages
                .iter()
                .map(
                    |ChatMessage {
                         sender, message, ..
                     }| {
                        let (b_statement, e_statement) = match sender {
                            Sender::Bot => ("", ""),
                            Sender::Human => ("[INST]", "[/INST]"),
                        };
                        format!("{b_statement}{message}{e_statement}")
                    },
                )
                .fold(String::new(), |x, y| format!("{x}\n{y}")),
        })?)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    Ok(notes)
}
