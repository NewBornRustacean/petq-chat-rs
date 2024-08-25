mod chat_stream;
use chat_stream::{chat_stream_handler, insert_to_db_from_queue,ChatParams, ChatRecord, __path_chat_stream_handler};
use axum::{response::IntoResponse, routing::get, Router};
use futures::StreamExt;
use mongodb::{Client as MongoClient, Collection};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
#[derive(OpenApi)]
#[openapi(paths(chat_stream_handler), components(schemas(ChatParams, ChatRecord)))]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<ChatRecord>(100);
    let chat_queue = Arc::new(Mutex::new(tx));
    // let db_client = MongoClient::with_uri_str(":memory:").await?;

    // Start the job consumer in a background task
    tokio::spawn(async move {
        if let Err(e) = insert_to_db_from_queue(rx).await {
            eprintln!("Failed to consume job queue: {}", e);
        }
    });

    let router = Router::new()
        .route("/chat-stream/:userid/:chatid", get(chat_stream_handler))
        .with_state(chat_queue)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));


    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();

    axum::serve(listener, router).await.unwrap();
    Ok(())
}

