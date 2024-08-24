use std::error::Error;

use async_openai::types::ChatCompletionRequestUserMessageArgs;
use async_openai::{types::CreateChatCompletionRequestArgs, Client};
use futures::StreamExt;
use axum::{response::IntoResponse, routing::get, Router, Json};
use axum::response::sse::{Sse, Event};
use axum::extract::Query;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;

#[derive(serde::Deserialize)]
pub struct ChatParams {
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let router = Router::new().route("/chat-stream", get(chat_stream_handler));
    let listener = tokio::net::TcpListener::bind("localhost:3000").await.unwrap();

    axum::serve(listener, router).await.unwrap();

    Ok(())
}

async fn chat_stream_handler(Query(params): Query<ChatParams>) -> impl IntoResponse {
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        if let Err(e) = generate_chat_stream(params.prompt, tx).await {
            println!("failed to stream chat: {}", e);
        }
    });

    let stream = ReceiverStream::new(rx);
    Sse::new(stream.map(|msg| Ok::<_, std::convert::Infallible>(Event::default().data(msg))))
}


async fn generate_chat_stream(prompt: String, tx: mpsc::Sender<String>) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let mut accumulated_content = String::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .max_tokens(512u32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(&*prompt)
            .build()?
            .into()])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                for chat_choice in response.choices.iter() {
                    if let Some(ref content) = chat_choice.delta.content {
                        accumulated_content.push_str(content);

                        if let Err(e) = tx.send(content.clone()).await {
                            eprintln!("failed to send content: {}", e);
                        }
                    }
                }
            }
            Err(err) => {
                if let Err(e) = tx.send(format!("error: {err}")).await {
                    eprintln!("failed to send error message: {}", e);
                }
            }
        }
    }
    println!("generated sentences: {:?}", accumulated_content);

    Ok(())
}