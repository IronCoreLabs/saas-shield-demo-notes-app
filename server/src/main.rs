use anyhow::Result;
use aws_sdk_s3 as s3;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{Request, State},
    http::{header::CONTENT_TYPE, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post, put},
    Router, ServiceExt,
};
use axum_extra::extract::CookieJar;
use db::{get_organization, OrganizationTable};
use elasticsearch::{
    http::transport::Transport,
    indices::{IndicesCreateParts, IndicesExistsParts},
    Elasticsearch,
};
use ironcore_alloy::{saas_shield::config::SaasShieldConfiguration, SaasShield};
use ollama_rs::Ollama;
use serde_json::json;
use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool},
    Sqlite,
};
use std::{str::FromStr, sync::Arc, time::Duration};
use tower::{layer::Layer, BoxError, ServiceBuilder};
use tower_http::{cors::CorsLayer, normalize_path::NormalizePathLayer, trace::TraceLayer};
use tracing::{debug, error, info, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    db: SqlitePool,
    sdk: Arc<SaasShield>,
    aws_sdk: s3::Client,
    es_sdk: Elasticsearch,
    ai_sdk: Ollama,
}
const DB_URL: &str = "sqlite://sqlite.db";
pub const INDEX_NAME: &str = "demo";
pub const SENTENCE_MODEL_NAME: &str = "all-minilm";
pub const CHATBOT_MODEL_NAME: &str = "llama-demo";

mod attachments;
mod categories;
mod db;
mod embeddings;
mod notes;
mod search_service;

async fn set_up_search_client() -> Result<Elasticsearch> {
    let transport = Transport::single_node("http://localhost:8675")?;
    let client = Elasticsearch::new(transport);
    info!("Trying to create search service index.");
    let index_exists_response = client
        .indices()
        .exists(IndicesExistsParts::Index(&[INDEX_NAME]))
        .send()
        .await?;

    // create index if it doesn't exist
    if index_exists_response.status_code() == StatusCode::NOT_FOUND {
        client
            .indices()
            .create(IndicesCreateParts::Index(INDEX_NAME))
            .body(json!({
                "mappings": {
                    "properties": {
                        "title_vector": {
                            "type": "dense_vector",
                            "dims": 384,
                            "index": "true",
                            "similarity": "cosine",
                        },
                        "body_vector": {
                            "type": "dense_vector",
                            "dims": 384,
                            "index": "true",
                            "similarity": "cosine",
                        }
                    }
                }
            }))
            .send()
            .await?
            .error_for_status_code()?;
        info!("Search service index created.");
    } else {
        info!("Search service index already exists.")
    }
    Ok(client)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        info!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => info!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        info!("Database already exists");
    }

    let options = SqliteConnectOptions::from_str(DB_URL)?.journal_mode(SqliteJournalMode::Delete);
    let db = SqlitePool::connect_with(options).await?;
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");

    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await?
        .run(&db)
        .await;

    match migration_results {
        Ok(_) => info!("Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }
    let api_key = std::env::var("TSP_API_KEY")?;
    let sdk_config = SaasShieldConfiguration::new(
        "http://localhost:32804".to_string(),
        api_key,
        true,
        Some(1.0),
    )?;
    let sdk = SaasShield::new(&sdk_config);
    let es_sdk = set_up_search_client().await?;

    let ai_sdk = Ollama::default();

    // This is the way to do path style with the aws sdk in rust
    let shared_config = aws_config::from_env()
        .endpoint_url("http://localhost:8080")
        .load()
        .await;
    let s3_config_builder: s3::config::Builder = (&shared_config).into();
    let final_config = s3_config_builder.force_path_style(true).build();
    let aws_sdk = s3::client::Client::from_conf(final_config);

    let state = AppState {
        db: db.clone(),
        sdk,
        aws_sdk,
        es_sdk,
        ai_sdk,
    };
    // Compose the routes
    let app = NormalizePathLayer::trim_trailing_slash().layer(
        Router::new()
            .route("/api/notes", get(notes::list).post(notes::create))
            .route("/api/attachments", post(attachments::create))
            .route("/api/notes/:id", get(notes::get).put(notes::update))
            .route("/api/notes/:id/rekey", put(notes::rekey))
            .route("/api/notes/search", post(notes::search))
            .route("/api/categories", get(categories::list))
            .route("/api/chat", post(notes::chat))
            // Add middleware to all routes
            .layer(
                ServiceBuilder::new()
                    .layer(
                        CorsLayer::new()
                            .allow_origin(["http://localhost:9002".parse::<HeaderValue>().unwrap()])
                            .allow_methods([Method::GET, Method::PUT, Method::POST])
                            .allow_headers([CONTENT_TYPE])
                            .allow_credentials(true),
                    )
                    .layer(HandleErrorLayer::new(|error: BoxError| async move {
                        if error.is::<tower::timeout::error::Elapsed>() {
                            Ok(StatusCode::REQUEST_TIMEOUT)
                        } else {
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Unhandled internal error: {error}"),
                            ))
                        }
                    }))
                    .timeout(Duration::from_secs(30))
                    .layer(TraceLayer::new_for_http())
                    .layer(middleware::from_fn_with_state(db.clone(), auth))
                    .into_inner(),
            )
            .with_state(state),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7654")
        .await
        .unwrap();
    debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .unwrap();
    Ok(())
}

#[derive(Debug, Clone)]
pub struct CurrentOrganization(pub OrganizationTable);
async fn auth(
    State(db): State<SqlitePool>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let jar = CookieJar::from_headers(req.headers());

    let org_login = if let Some(org_login) = jar.get("organization") {
        trace!("Found cookie {:?}", &org_login);
        org_login.value()
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    match get_organization(&db, org_login).await {
        Ok(Some(current_user)) => {
            // insert the current user into a request extension so the handler can
            // extract it
            req.extensions_mut()
                .insert(CurrentOrganization(current_user));
            Ok(next.run(req).await)
        }
        Ok(None) => {
            info!("Organization login for '{}' was not found.", org_login);
            Err(StatusCode::UNAUTHORIZED)
        }
        Err(e) => {
            error!(
                "Looking up organization '{}' caused an error: '{:?}'",
                org_login, e
            );
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
