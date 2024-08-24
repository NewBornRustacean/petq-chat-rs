mod chat_stream;
use chat_stream::chat_stream_handler;
use std::error::Error;
use futures::StreamExt;
use axum::{response::IntoResponse, routing::get, Router};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let router = Router::new().route("/chat-stream", get(chat_stream_handler));
    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();

    axum::serve(listener, router).await.unwrap();

    Ok(())
}
