use crate::{
    attachments::{AttachmentInfo, CreateAttachmentRequest, CreateAttachmentResponse},
    notes::{CreateNoteRequest, UpdateNoteRequest},
    CurrentOrganization,
};
use anyhow::{anyhow, Result};
use aws_sdk_s3::presigning::PresigningConfig;
use base64::{engine::general_purpose::STANDARD, Engine};
use futures::future::join_all;
use ironcore_alloy::{
    deterministic::{DeterministicFieldOps, EncryptedField, EncryptedFields, PlaintextField},
    standard::{EdekWithKeyIdHeader, EncryptedDocument, PlaintextDocument, StandardDocumentOps},
    AlloyMetadata, DerivationPath, EncryptedBytes, FieldId, PlaintextBytes, SaasShield, SecretPath,
    TenantId,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    Sqlite, SqlitePool, Transaction,
};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[derive(Clone, Debug, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct EncryptedString(pub String);

impl EncryptedString {
    pub fn new(bytes: EncryptedBytes) -> EncryptedString {
        EncryptedString(STANDARD.encode(bytes.0))
    }

    pub fn to_enc_bytes(self) -> Result<EncryptedBytes> {
        STANDARD
            .decode(self.0)
            .map(|x| EncryptedBytes(x))
            .map_err(|x| x.into())
    }
}

#[derive(Clone, Debug, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct DeterministicallyEncryptedString(String);

impl DeterministicallyEncryptedString {
    fn new(bytes: EncryptedBytes) -> DeterministicallyEncryptedString {
        DeterministicallyEncryptedString(STANDARD.encode(bytes.0))
    }

    fn to_enc_bytes(self) -> Result<EncryptedBytes> {
        STANDARD
            .decode(self.0)
            .map(|x| EncryptedBytes(x))
            .map_err(|x| x.into())
    }
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct AttachmentTable {
    pub id: u32,
    pub note_id: Option<u32>,
    pub filename: String,
    pub created: String,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct NoteTable {
    pub id: u32,
    pub org_id: u32,
    pub category: Option<DeterministicallyEncryptedString>,
    #[sqlx(rename = "title")]
    pub enc_title: EncryptedString,
    #[sqlx(rename = "body")]
    pub enc_body: EncryptedString,
    pub edek: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Default, FromRow, Serialize)]
pub struct Note {
    pub id: u32,
    pub category: Option<String>,
    pub title: String,
    pub body: String,
    pub created: String,
    pub updated: String,
    pub attachments: Vec<AttachmentInfo>,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct OrganizationTable {
    pub id: u32,
    pub login: String,
    pub name: String,
    pub created: String,
    pub updated: String,
}

pub struct EncryptedNote {
    pub title: EncryptedString,
    pub body: EncryptedString,
    pub category: Option<DeterministicallyEncryptedString>,
    pub edek: String,
}

#[derive(Debug, Type, Deserialize, FromRow)]
pub struct OnlyCategory {
    category: DeterministicallyEncryptedString,
}

#[derive(Debug, Type, Deserialize, FromRow)]
pub struct OnlyEdek {
    pub edek: EncryptedString,
}

async fn create_attachment_vec(
    aws_sdk: aws_sdk_s3::Client,
    org: &CurrentOrganization,
    attachment_tables: Vec<AttachmentTable>,
) -> Result<Vec<AttachmentInfo>> {
    join_all(attachment_tables.into_iter().map(|attachment| {
        create_attachment_info(aws_sdk.clone(), org, attachment.filename, attachment.id)
    }))
    .await
    .into_iter()
    .collect::<Result<_, _>>()
}

async fn create_attachment_info(
    aws_sdk: aws_sdk_s3::Client,
    org: &CurrentOrganization,
    filename: String,
    attachment_id: u32,
) -> Result<AttachmentInfo> {
    let content_type = if filename.ends_with(".jpg") {
        Some("image/jpeg".to_string())
    } else {
        None
    };
    let presigned_request = aws_sdk
        .get_object()
        .bucket("icl-demo-notes-app")
        .key(format!("{}/{}-{}", org.0.login, attachment_id, filename))
        .set_response_content_type(content_type)
        .presigned(PresigningConfig::expires_in(Duration::from_secs(9999))?)
        .await?;

    Ok(AttachmentInfo {
        id: attachment_id,
        filename: filename.to_string(),
        url: presigned_request.uri().to_string(),
    })
}

async fn update_attachments(
    trx: &mut Transaction<'static, Sqlite>,
    attachments_to_update: Vec<u32>,
    note_id: u32,
) -> Result<Vec<AttachmentTable>> {
    let mut updated_attachments: Vec<AttachmentTable> =
        Vec::with_capacity(attachments_to_update.len());
    for attachment_id in attachments_to_update.iter() {
        let res = sqlx::query_as::<_, AttachmentTable>(
            "UPDATE attachment SET note_id = $1 WHERE id = $2 RETURNING *",
        )
        .bind(note_id)
        .bind(attachment_id)
        .fetch_optional(&mut **trx)
        .await?;

        if let Some(attachment) = res {
            updated_attachments.push(attachment);
        };
    }
    Ok(updated_attachments)
}

async fn get_attachments_and_create_info(
    note: Note,
    org: &CurrentOrganization,
    pool: &SqlitePool,
    aws_sdk: aws_sdk_s3::Client,
) -> Result<Note> {
    let mut conn = pool.acquire().await?;
    let attachment_tables =
        sqlx::query_as::<_, AttachmentTable>("SELECT * FROM attachment WHERE note_id = $1")
            .bind(note.id)
            .fetch_all(&mut *conn)
            .await?;

    let attachments = create_attachment_vec(aws_sdk, org, attachment_tables).await?;

    Ok(Note {
        attachments,
        ..note
    })
}

async fn encrypt_note(
    note: CreateNoteRequest,
    organization: CurrentOrganization,
    sdk: Arc<SaasShield>,
) -> Result<EncryptedNote> {
    let metadata = AlloyMetadata::new_simple(TenantId(organization.0.login));
    let plaintext_document = PlaintextDocument(
        [
            (
                FieldId("title".to_string()),
                PlaintextBytes(note.title.into_bytes()),
            ),
            (
                FieldId("body".to_string()),
                PlaintextBytes(note.body.into_bytes()),
            ),
        ]
        .into(),
    );
    let mut encrypted = sdk
        .standard()
        .encrypt(plaintext_document, &metadata)
        .await?;

    let enc_category = encrypt_category(note.category, sdk, &metadata).await?;
    let enc_title = encrypted
        .document
        .remove(&FieldId("title".to_string()))
        .ok_or(anyhow!("ironcore_alloy didn't encrypt this field"))?;
    let enc_body = encrypted
        .document
        .remove(&FieldId("body".to_string()))
        .ok_or(anyhow!("ironcore_alloy didn't encrypt this field"))?;
    let edek = STANDARD.encode(encrypted.edek.0);
    Ok(EncryptedNote {
        title: EncryptedString::new(enc_title),
        body: EncryptedString::new(enc_body),
        category: enc_category,
        edek,
    })
}

async fn encrypt_category(
    category: Option<String>,
    sdk: Arc<SaasShield>,
    metadata: &AlloyMetadata,
) -> Result<Option<DeterministicallyEncryptedString>> {
    let plaintext_category = category.map(|category| PlaintextField {
        plaintext_field: PlaintextBytes(category.into_bytes()),
        secret_path: SecretPath("".to_string()),
        derivation_path: DerivationPath("note/category".to_string()),
    });
    let result = match plaintext_category {
        Some(category) => Some(DeterministicallyEncryptedString::new(
            sdk.deterministic()
                .encrypt(category, &metadata)
                .await?
                .encrypted_field,
        )),
        None => None,
    };
    Ok(result)
}

async fn decrypt_note(
    row: NoteTable,
    sdk: Arc<SaasShield>,
    metadata: &AlloyMetadata,
) -> Result<Note> {
    let enc_document = EncryptedDocument {
        edek: EdekWithKeyIdHeader(EncryptedBytes(STANDARD.decode(row.edek)?)),
        document: [
            (FieldId("title".to_string()), row.enc_title.to_enc_bytes()?),
            (FieldId("body".to_string()), row.enc_body.to_enc_bytes()?),
        ]
        .into(),
    };
    let mut decrypted = sdk.standard().decrypt(enc_document, &metadata).await?;
    let dec_title = decrypted
        .0
        .remove(&FieldId("title".to_string()))
        .ok_or(anyhow!("ironcore_alloy didn't decrypt this field"))?;
    let dec_body = decrypted
        .0
        .remove(&FieldId("body".to_string()))
        .ok_or(anyhow!("ironcore_alloy didn't decrypt this field"))?;
    let dec_category = match row.category {
        Some(category) => Some(String::from_utf8(
            sdk.deterministic()
                .decrypt(
                    EncryptedField {
                        encrypted_field: category.to_enc_bytes()?,
                        secret_path: SecretPath("".to_string()),
                        derivation_path: DerivationPath("note/category".to_string()),
                    },
                    &metadata,
                )
                .await?
                .plaintext_field
                .0,
        )?),
        None => None,
    };

    Ok(Note {
        id: row.id,
        category: dec_category,
        title: String::from_utf8(dec_title.0)?,
        body: String::from_utf8(dec_body.0)?,
        created: row.created,
        updated: row.updated,
        attachments: vec![],
    })
}

pub async fn create_attachment(
    pool: &SqlitePool,
    attachment: CreateAttachmentRequest,
    org: &CurrentOrganization,
    aws_sdk: aws_sdk_s3::Client,
) -> Result<CreateAttachmentResponse> {
    let mut trx = pool.begin().await?;
    let new_attachment = sqlx::query_as::<_, AttachmentTable>(
        "INSERT INTO attachment (filename) VALUES ($1) RETURNING *",
    )
    .bind(attachment.filename)
    .fetch_one(&mut *trx)
    .await?;
    trx.commit().await?;

    let presigned_request = aws_sdk
        .put_object()
        .bucket("icl-demo-notes-app")
        .key(format!(
            "{}/{}-{}",
            org.0.login, new_attachment.id, new_attachment.filename
        ))
        .presigned(PresigningConfig::expires_in(Duration::from_secs(9999))?)
        .await?;

    let info =
        create_attachment_info(aws_sdk, org, new_attachment.filename, new_attachment.id).await?;

    Ok(CreateAttachmentResponse {
        filename: info.filename,
        id: info.id,
        note_id: new_attachment.note_id,
        presigned_put_url: presigned_request.uri().to_string(),
        url: info.url,
    })
}

pub async fn create_note(
    pool: &SqlitePool,
    note: CreateNoteRequest,
    organization: &CurrentOrganization,
    sdk: Arc<SaasShield>,
    aws_sdk: aws_sdk_s3::Client,
) -> Result<Note> {
    let mut trx = pool.begin().await?;
    let encrypted_note = encrypt_note(note.clone(), organization.clone(), sdk).await?;
    let res = sqlx::query_as::<_, NoteTable>(
        "INSERT INTO note (org_id, title, body, category, edek) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(organization.0.id)
    .bind(encrypted_note.title)
    .bind(encrypted_note.body)
    .bind(encrypted_note.category)
    .bind(encrypted_note.edek)
    .fetch_one(&mut *trx)
    .await?;

    let mut updated_attachments: Vec<AttachmentTable> = Vec::with_capacity(note.attachments.len());
    for attachment_id in note.attachments.iter() {
        let res = sqlx::query_as::<_, AttachmentTable>(
            "UPDATE attachment SET note_id = $1 WHERE id = $2 RETURNING *",
        )
        .bind(res.id)
        .bind(attachment_id)
        .fetch_one(&mut *trx)
        .await?;

        updated_attachments.push(res);
    }
    trx.commit().await?;

    let attachments = create_attachment_vec(aws_sdk, organization, updated_attachments).await?;

    Ok(Note {
        id: res.id,
        category: note.category,
        title: note.title,
        body: note.body,
        created: res.created,
        updated: res.updated,
        attachments,
    })
}

pub async fn update_note(
    pool: &SqlitePool,
    note: UpdateNoteRequest,
    id: u32,
    organization: &CurrentOrganization,
    sdk: Arc<SaasShield>,
    aws_sdk: aws_sdk_s3::Client,
) -> Result<Note> {
    let mut trx = pool.begin().await?;
    let encrypted_note = encrypt_note(note.clone(), organization.clone(), sdk).await?;
    let res = sqlx::query_as::<_, NoteTable>(
        "UPDATE note SET title = $1, body = $2, category = $3, edek = $4, updated = (SELECT current_timestamp) WHERE id = $5 AND org_id = $6 RETURNING *",
    )
    .bind(encrypted_note.title)
    .bind(encrypted_note.body)
    .bind(encrypted_note.category)
    .bind(encrypted_note.edek)
    .bind(id)
    .bind(organization.0.id)
    .fetch_one(&mut *trx)
    .await?;

    // clear all the existing attachments
    sqlx::query("UPDATE attachment SET note_id = NULL WHERE note_id=$1")
        .bind(res.id)
        .execute(&mut *trx)
        .await?;

    let updated_attachments = update_attachments(&mut trx, note.attachments, res.id).await?;

    trx.commit().await?;

    let attachments: Vec<AttachmentInfo> =
        create_attachment_vec(aws_sdk, organization, updated_attachments).await?;

    Ok(Note {
        id: res.id,
        category: note.category,
        title: note.title,
        body: note.body,
        created: res.created,
        updated: res.updated,
        attachments,
    })
}

pub async fn get_note(
    pool: &SqlitePool,
    id: u32,
    organization: &CurrentOrganization,
    sdk: Arc<SaasShield>,
    aws_sdk: aws_sdk_s3::Client,
) -> Result<Option<Note>> {
    let mut conn = pool.acquire().await?;
    match sqlx::query_as::<_, NoteTable>("SELECT * FROM note WHERE id = $1 AND org_id = $2")
        .bind(id)
        .bind(organization.0.id)
        .fetch_optional(&mut *conn)
        .await?
    {
        Some(row) => {
            let decrypted_note = decrypt_note(
                row,
                sdk.clone(),
                &AlloyMetadata::new_simple(TenantId(organization.0.login.clone())),
            )
            .await?;
            get_attachments_and_create_info(decrypted_note, organization, pool, aws_sdk)
                .await
                .map(|note| Some(note))
        }
        None => Ok(None),
    }
}

pub async fn get_edek(
    pool: &SqlitePool,
    id: u32,
    organization: &CurrentOrganization,
) -> Result<Option<OnlyEdek>> {
    let mut conn = pool.acquire().await?;
    Ok(
        sqlx::query_as::<_, OnlyEdek>("SELECT edek FROM note WHERE id = $1 AND org_id = $2")
            .bind(id)
            .bind(organization.0.id)
            .fetch_optional(&mut *conn)
            .await?,
    )
}

pub async fn put_edek(
    pool: &SqlitePool,
    id: u32,
    organization: &CurrentOrganization,
    edek: EncryptedString,
) -> Result<u32> {
    let mut conn = pool.acquire().await?;

    let result = sqlx::query("UPDATE note SET edek=$1 WHERE id = $2 AND org_id = $3")
        .bind(edek.0)
        .bind(id)
        .bind(organization.0.id)
        .execute(&mut *conn)
        .await?;
    Ok(result.rows_affected() as u32)
}

pub async fn list_notes(
    pool: &SqlitePool,
    org: CurrentOrganization,
    sdk: Arc<SaasShield>,
    category: Option<String>,
    aws_sdk: aws_sdk_s3::Client,
) -> Result<Vec<Note>> {
    let metadata = AlloyMetadata::new_simple(TenantId(org.0.login.clone()));
    let mut conn = pool.acquire().await?;
    let query = match encrypt_category(category, sdk.clone(), &metadata).await? {
        Some(cat) => {
            sqlx::query_as::<_, NoteTable>(
            "SELECT * FROM note WHERE note.org_id=$1 AND note.category IS NOT NULL AND note.category=$2",
        )
        .bind(org.0.id)
        .bind(cat.0)
        }
        None => {
            sqlx::query_as::<_, NoteTable>("SELECT * FROM note WHERE note.org_id=$1").bind(org.0.id)
        }
    };

    let db_result = query.fetch_all(&mut *conn).await?;
    // TODO This really should do something smarter than this. We should use 2 batch calls total instead of doing 2 calls per note. This is ok for the demo though.
    let decrypted_notes: Vec<Note> = join_all(
        db_result
            .into_iter()
            .map(|row| decrypt_note(row, sdk.clone(), &metadata)),
    )
    .await
    .into_iter()
    .collect::<Result<_>>()?;

    let result = join_all(
        decrypted_notes
            .into_iter()
            .map(|note| get_attachments_and_create_info(note, &org, pool, aws_sdk.clone())),
    )
    .await
    .into_iter()
    .collect::<Result<_>>()?;
    Ok(result)
}

pub async fn search_notes(
    pool: &SqlitePool,
    ids: Vec<u32>,
    org: &CurrentOrganization,
    sdk: Arc<SaasShield>,
    aws_sdk: aws_sdk_s3::Client,
) -> Result<Vec<Note>> {
    let mut conn = pool.acquire().await?;

    let parameters = ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", ");
    let sql = format!(
        "SELECT * FROM note WHERE note.org_id=? AND note.id IN ({})",
        parameters
    );
    let mut query = sqlx::query_as::<_, NoteTable>(&sql).bind(org.0.id);
    for id in &ids {
        query = query.bind(id);
    }

    let db_result = query.fetch_all(&mut *conn).await?;
    let metadata = AlloyMetadata::new_simple(TenantId(org.0.login.clone()));
    // TODO This really should do something smarter than this. We should use 2 batch calls total instead of doing 2 calls per note. This is ok for the demo though.
    let decrypted_notes = join_all(
        db_result
            .into_iter()
            .map(|row| decrypt_note(row, sdk.clone(), &metadata)),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?;
    let result = join_all(
        decrypted_notes
            .into_iter()
            .map(|note| get_attachments_and_create_info(note, &org, pool, aws_sdk.clone())),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?;
    let mut hashmap_result = result
        .into_iter()
        .map(|note| (note.id, note))
        .collect::<HashMap<_, _>>();
    let sorted_result = ids
        .into_iter()
        .flat_map(|id| hashmap_result.remove(&id))
        .collect();
    Ok(sorted_result)
}

pub async fn get_organization(
    pool: &SqlitePool,
    login: &str,
) -> Result<Option<OrganizationTable>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query_as::<_, OrganizationTable>("SELECT * FROM organization WHERE login = $1")
        .bind(login)
        .fetch_optional(&mut *conn)
        .await
}

pub async fn list_categories(
    pool: &SqlitePool,
    org: CurrentOrganization,
    sdk: Arc<SaasShield>,
) -> Result<Vec<String>> {
    let mut conn = pool.acquire().await?;
    let result = sqlx::query_as::<_, OnlyCategory>(
        "SELECT DISTINCT n.category FROM organization AS o JOIN note AS n ON n.org_id = o.id WHERE o.id=$1 AND n.category IS NOT NULL",
    )
    .bind(org.0.id)
    .fetch_all(&mut *conn)
    .await?;

    let encrypted_fields: HashMap<_, _> = (0..)
        .zip(result.into_iter().map(|c| c.category))
        .map(|(n, cat)| -> Result<_> { Ok((FieldId(n.to_string()), create_encrypted_field(cat)?)) })
        .collect::<Result<_, _>>()?;
    let decrypted = sdk
        .deterministic()
        .decrypt_batch(
            EncryptedFields(encrypted_fields),
            &AlloyMetadata::new_simple(TenantId(org.0.login)),
        )
        .await?;
    let decrypted_strings = decrypted
        .successes
        .0
        .into_iter()
        .map(|(_, field_value)| String::from_utf8(field_value.plaintext_field.0).unwrap())
        .sorted()
        .collect();
    Ok(decrypted_strings)
}

fn create_encrypted_field(d: DeterministicallyEncryptedString) -> Result<EncryptedField> {
    Ok(EncryptedField {
        encrypted_field: d.to_enc_bytes()?,
        secret_path: SecretPath("".to_string()),
        derivation_path: DerivationPath("note/category".to_string()),
    })
}
