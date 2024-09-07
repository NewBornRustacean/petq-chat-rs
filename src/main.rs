mod chat_stream;
mod memory;

use async_openai::Client as OpenaiClient;
use axum::{response::IntoResponse, routing::get, Router};
use chat_stream::{chat_stream_handler, ChatRecord, ChatParams, PetProfileDB, PetProfile, SharedState, __path_chat_stream_handler};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::cors::{CorsLayer, Any};
use serde::Deserialize;
use std::fs;
use uuid::{uuid, Uuid};

#[derive(OpenApi)]
#[openapi(paths(chat_stream_handler), components(schemas(ChatParams, ChatRecord)))]
struct ApiDoc;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub llm_to_use: String,
    pub max_tokens: u32,
    pub sliding_window_size: usize,
    pub client_url: String,
}

impl AppConfig {
    // Load configuration from a `config.toml` file
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_contents = fs::read_to_string(file_path)?;
        let config: AppConfig = toml::from_str(&config_contents)?;
        Ok(config)
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<(), Box<dyn Error>> {
    let chat_stream_config = AppConfig::from_file("src/chat_stream_config.toml")?;

    let shared_state = SharedState {
        chat_collection: Arc::new(Mutex::new(HashMap::new())),
        openai_client: Arc::new(OpenaiClient::new()), // Initialize your OpenAI client here
        pet_profile_db: Arc::new(Mutex::new(HashMap::new())), // Your pet profile "DB"
    };

    // Configure CORS
    let cors_layer = CorsLayer::new()
        // Allow `GET` and `POST` when accessing the resource
        .allow_methods([http::Method::GET, http::Method::POST])
        // Allow requests from any origin
        .allow_origin(Any)
        // Allow sending any headers
        .allow_headers(Any);

    {
        let mut db = shared_state.pet_profile_db.lock().await;
        db.insert(
            uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            PetProfile {
                name: "설렘".to_string(),
                species: "강아지, 포메라니안".to_string(),
                age: "1".to_string(),
                health_concerns: "중성화 수술 완료. 닭고기 알러지 있음.".to_string(),
            },
        );
    }

    // Set up the Axum router with shared HashMap and OpenAI client
    let router = Router::new()
        .route("/chat-stream/:userid/:chatid", get(chat_stream_handler))
        .with_state(shared_state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors_layer);

    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
