mod chat_stream;
use axum::{response::IntoResponse, routing::get, Router};
use chat_stream::{chat_stream_handler, ChatRecord};
use futures::StreamExt;
use mongodb::{Client as MongoClient, Collection};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, _rx) = mpsc::channel::<ChatRecord>(100);
    let chat_queue = Arc::new(Mutex::new(tx));

    let router = Router::new()
        .route("/chat-stream/:userid/:chatid", get(chat_stream_handler))
        .with_state(chat_queue);

    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();

    axum::serve(listener, router).await.unwrap();

    // let db_client = MongoClient::with_uri_str("mongodb://localhost:27017").await?;

    Ok(())
}
