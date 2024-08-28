mod chat_stream;

use async_openai::Client as OpenAIClient;
use axum::{response::IntoResponse, routing::get, Router};
use chat_stream::{
    chat_stream_handler, AppState, ChatParams, __path_chat_stream_handler,
};
use futures::StreamExt;
use mongodb::{Client as MongoClient, Collection};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(chat_stream_handler),
    components(schemas(ChatParams))
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let openai_config
    let openai_client = OpenAIClient::new();
    let app_state = Arc::new(AppState {
        openai_client: openai_client,
    });

    let router = Router::new()
        .route("/chat-stream/:userid/:chatid", get(chat_stream_handler))
        .with_state(app_state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, router).await.unwrap();

    Ok(())
}
