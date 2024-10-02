use crate::{
    db::Note,
    notes::{CreateNoteRequest, QueryChatbotRequest, SearchNoteRequest},
    search_service::Knn,
    CurrentOrganization, CHATBOT_MODEL_NAME, SENTENCE_MODEL_NAME,
};
use anyhow::Result;
use ironcore_alloy::{
    vector::{EncryptedVector, PlaintextVector, PlaintextVectors, VectorId, VectorOps},
    AlloyMetadata, DerivationPath, SaasShield, SecretPath, TenantId,
};
use itertools::Itertools;
use ollama_rs::{
    generation::{
        completion::request::GenerationRequest, embeddings::request::GenerateEmbeddingsRequest,
    },
    Ollama,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug)]
pub struct EncryptedEmbeddings {
    pub enc_title: EncryptedVector,
    pub enc_body: EncryptedVector,
}

#[derive(Debug, Serialize)]
pub struct QueryChatbotResponse {
    pub response: String,
    pub note_id: u32,
}

pub async fn generate_and_encrypt_embedding(
    ai_sdk: Ollama,
    sdk: Arc<SaasShield>,
    note: CreateNoteRequest,
    organization: &CurrentOrganization,
) -> Result<EncryptedEmbeddings> {
    let metadata = AlloyMetadata::new_simple(TenantId(organization.0.login.clone()));
    let request = GenerateEmbeddingsRequest::new(
        SENTENCE_MODEL_NAME.to_string(),
        vec![note.body, note.title].into(),
    );
    let embedding = ai_sdk.generate_embeddings(request).await?.embeddings;
    let plaintext_vectors = ["body", "title"]
        .into_iter()
        .zip(embedding)
        .map(|(k, v)| {
            (
                VectorId(k.to_string()),
                PlaintextVector {
                    plaintext_vector: v,
                    secret_path: SecretPath("".to_string()),
                    derivation_path: DerivationPath("note/embedding".to_string()),
                },
            )
        })
        .collect();
    let mut encrypted_embeddings = sdk
        .vector()
        .encrypt_batch(PlaintextVectors(plaintext_vectors), &metadata)
        .await?;
    let enc_body = encrypted_embeddings
        .successes
        .0
        .remove(&VectorId("body".to_string()))
        .unwrap();
    let enc_title = encrypted_embeddings
        .successes
        .0
        .remove(&VectorId("title".to_string()))
        .unwrap();
    Ok(EncryptedEmbeddings {
        enc_title,
        enc_body,
    })
}

pub async fn generate_query_embeddings(
    ai_sdk: Ollama,
    sdk: Arc<SaasShield>,
    search: SearchNoteRequest,
    organization: &CurrentOrganization,
) -> Result<Vec<Knn>> {
    let metadata = AlloyMetadata::new_simple(TenantId(organization.0.login.clone()));
    let input = search.title.into_iter().chain(search.body).collect_vec();
    let request = GenerateEmbeddingsRequest::new(SENTENCE_MODEL_NAME.to_string(), input.into());
    let embedding = ai_sdk.generate_embeddings(request).await?.embeddings;
    let plaintext_vectors = ["title", "body"]
        .into_iter()
        .zip(embedding)
        .map(|(field_name, vector)| {
            (
                VectorId(field_name.to_string()),
                PlaintextVector {
                    plaintext_vector: vector,
                    secret_path: SecretPath("".to_string()),
                    derivation_path: DerivationPath("note/embedding".to_string()),
                },
            )
        })
        .collect();
    Ok(sdk
        .vector()
        .generate_query_vectors(PlaintextVectors(plaintext_vectors), &metadata)
        .await?
        .0
        .into_iter()
        .map(|(field_name, mut vector)| Knn {
            field: format!("{}_vector", field_name.0),
            query_vector: vector.remove(0).encrypted_vector, // we won't be in rotation for the demo
            num_candidates: 15,
            k: 3,
            boost: 0.8,
        })
        .collect_vec())
}

pub async fn query_chatbot(
    ai_sdk: Ollama,
    note: Note,
    request: QueryChatbotRequest,
) -> Result<QueryChatbotResponse> {
    let prompt = format!(
        r###"
        note:
        {}: {}

        question:
        {}
        "###,
        note.title, note.body, request.question
    );
    let res = ai_sdk
        .generate(GenerationRequest::new(
            CHATBOT_MODEL_NAME.to_string(),
            prompt,
        ))
        .await?;
    Ok(QueryChatbotResponse {
        response: res.response,
        note_id: note.id,
    })
}
