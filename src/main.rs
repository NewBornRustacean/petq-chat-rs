mod chat_stream;
use async_openai::Client as OpenaiClient;
use axum::{response::IntoResponse, routing::get, Router};
use chat_stream::{chat_stream_handler, ChatRecord, ChatParams, __path_chat_stream_handler};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::cors::{CorsLayer, Any};

#[derive(OpenApi)]
#[openapi(paths(chat_stream_handler), components(schemas(ChatParams, ChatRecord)))]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let chat_collection: Arc<Mutex<HashMap<String, ChatRecord>>> = Arc::new(Mutex::new(HashMap::new()));

    let openai_client = Arc::new(OpenaiClient::new());

    // Configure CORS
    let cors_layer = CorsLayer::new()
        // Allow `GET` and `POST` when accessing the resource
        .allow_methods([http::Method::GET, http::Method::POST])
        // Allow requests from any origin
        .allow_origin(Any)
        // Allow sending any headers
        .allow_headers(Any);

    // Set up the Axum router with shared HashMap and OpenAI client
    let router = Router::new()
        .route("/chat-stream/:userid/:chatid", get(chat_stream_handler))
        .with_state((chat_collection, openai_client))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors_layer);

    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
