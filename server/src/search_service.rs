use crate::{
    embeddings::EncryptedEmbeddings,
    notes::{CreateNoteRequest, UpdateNoteRequest},
    CurrentOrganization, INDEX_NAME,
};
use anyhow::Result;
use elasticsearch::{Elasticsearch, IndexParts, SearchParts};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct SearchServiceNote {
    pub org_id: String,
    pub title: String,
    pub body: String,
    pub title_vector: Vec<f32>,
    pub body_vector: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct QueryResponse {
    hits: HitsObject,
}
impl QueryResponse {
    fn get_ids(self) -> Result<Vec<u32>> {
        Ok(self
            .hits
            .hits
            .into_iter()
            .flat_map(|hit| hit.id.as_str().and_then(|u| u.parse::<u32>().ok()))
            .collect_vec())
    }
}
#[derive(Debug, Deserialize)]
struct HitsObject {
    hits: Vec<NoteId>,
}
#[derive(Debug, Deserialize)]
struct NoteId {
    #[serde(rename = "_id")]
    id: Value,
}

pub enum QueryType {
    Keyword {
        title: Option<String>,
        body: Option<String>,
    },
    Knn {
        embeddings: Vec<Knn>,
    },
}

impl QueryType {
    pub fn make_query(self, org_id: String) -> OuterQuery {
        let should = match self {
            QueryType::Keyword { title, body } => title
                .map(|title| ("title".to_string(), title))
                .into_iter()
                .chain(body.map(|body| ("body".to_string(), body)))
                .map(|(key, value)| Should {
                    r#match: Some([(key, value)].into()),
                    knn: None,
                })
                .collect_vec(),
            QueryType::Knn { embeddings } => embeddings
                .into_iter()
                .map(|knn| Should {
                    r#match: None,
                    knn: Some(knn),
                })
                .collect_vec(),
        };
        OuterQuery {
            query: Query {
                bool: Bool {
                    filter: Some(Filter {
                        term: Term { org_id },
                    }),
                    must: Some(Box::new(Query {
                        bool: Bool {
                            filter: None,
                            must: None,
                            should,
                        },
                    })),
                    should: vec![],
                },
            },
        }
    }
}

pub async fn index_note(
    note_id: u32,
    request: CreateNoteRequest,
    organization: &CurrentOrganization,
    search_client: Elasticsearch,
    embeddings: EncryptedEmbeddings,
) -> Result<()> {
    let search_service_note = SearchServiceNote {
        org_id: organization.0.login.clone(),
        title: request.title,
        body: request.body,
        title_vector: embeddings.enc_title.encrypted_vector,
        body_vector: embeddings.enc_body.encrypted_vector,
    };
    let note_id_str = note_id.to_string();
    search_client
        .index(IndexParts::IndexId(INDEX_NAME, &note_id_str))
        .body(search_service_note)
        .send()
        .await?
        .error_for_status_code()?;
    Ok(())
}

/// Deletes the existing note then inserts the new one with the same ID.
/// This doesn't do any checking that the provided ID is in the given org,
/// so this is intended to be called after the database function which does check.
pub async fn update_note(
    note_id: u32,
    request: UpdateNoteRequest,
    organization: &CurrentOrganization,
    search_client: Elasticsearch,
    embeddings: EncryptedEmbeddings,
) -> Result<()> {
    let note_id_str = note_id.to_string();
    search_client
        .delete(elasticsearch::DeleteParts::IndexId(
            INDEX_NAME,
            &note_id_str,
        ))
        .send()
        .await?
        .error_for_status_code()?;
    index_note(note_id, request, organization, search_client, embeddings).await?;
    Ok(())
}

pub async fn query_notes(
    organization: &CurrentOrganization,
    search_client: Elasticsearch,
    query_type: QueryType,
) -> Result<Vec<u32>> {
    let query = query_type.make_query(organization.0.login.clone());
    let search_res = search_client
        .search(SearchParts::Index(&[INDEX_NAME]))
        .body(query)
        .send()
        .await?
        .error_for_status_code()?
        .json::<QueryResponse>()
        .await?;
    search_res.get_ids()
}

#[derive(Clone, Debug, Serialize)]
pub struct OuterQuery {
    query: Query,
}
#[derive(Clone, Debug, Serialize)]
pub struct Query {
    bool: Bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct Bool {
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Filter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    must: Option<Box<Query>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    should: Vec<Should>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Filter {
    term: Term,
}
#[derive(Clone, Debug, Serialize)]
pub struct Term {
    #[serde(rename = "org_id.keyword")]
    org_id: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Should {
    #[serde(skip_serializing_if = "Option::is_none")]
    r#match: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    knn: Option<Knn>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Knn {
    pub field: String,
    pub query_vector: Vec<f32>,
    pub num_candidates: u32,
    pub k: u32,
    pub boost: f32,
}
