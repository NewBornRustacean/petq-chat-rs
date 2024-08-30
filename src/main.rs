mod chat_stream;
use async_openai::Client as OpenaiClient;
use axum::{response::IntoResponse, routing::get, Router};
use chat_stream::{chat_stream_handler, ChatRecord, ChatParams, __path_chat_stream_handler};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(chat_stream_handler), components(schemas(ChatParams, ChatRecord)))]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let chat_collection: Arc<Mutex<HashMap<String, ChatRecord>>> = Arc::new(Mutex::new(HashMap::new()));

    let openai_client = Arc::new(OpenaiClient::new());

    // Set up the Axum router with shared HashMap and OpenAI client
    let router = Router::new()
        .route("/chat-stream/:userid/:chatid", get(chat_stream_handler))
        .with_state((chat_collection, openai_client))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
