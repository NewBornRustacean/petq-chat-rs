use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client as OpenaiClient,
};
use axum::{
    extract::{Path, Query, State},
    response::sse::Event,
    response::{IntoResponse, Sse},
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct ChatParams {
    prompt: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ChatRecord {
    userid: Uuid,
    chatid: Uuid,
    prompt: String,
    response: String,
}

type SharedState = (Arc<Mutex<HashMap<String, ChatRecord>>>, Arc<OpenaiClient<OpenAIConfig>>);

#[utoipa::path(
    get,
    path = "/chat-stream/{userid}/{chatid}",
    params(
        ("userid" = Uuid, Path, description = "User ID"),
        ("chatid" = Uuid, Path, description = "Chat ID"),
        ChatParams
    ),
    responses(
        (status = 200, description = "Successful chat stream", body = String),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn chat_stream_handler(
    Path((userid, chatid)): Path<(Uuid, Uuid)>,
    Query(params): Query<ChatParams>,
    State((chat_collection, openai_client)): State<SharedState>,
) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    let chat_collection_clone = Arc::clone(&chat_collection);
    let openai_client_clone = Arc::clone(&openai_client);

    tokio::spawn(async move {
        if let Err(e) = generate_chat_stream(
            params.prompt.as_str(),
            tx,
            chat_collection_clone,
            userid,
            chatid,
            openai_client_clone,
        )
        .await
        {
            eprintln!("Failed to stream chat: {}", e);
        }
    });

    let stream = ReceiverStream::new(rx);
    Sse::new(stream.map(|msg| Ok::<_, std::convert::Infallible>(Event::default().data(msg))))
}

async fn generate_chat_stream(
    prompt: &str,
    tx: tokio::sync::mpsc::Sender<String>,
    chat_collection: Arc<Mutex<HashMap<String, ChatRecord>>>,
    userid: Uuid,
    chatid: Uuid,
    openai_client: Arc<OpenaiClient<OpenAIConfig>>,
) -> Result<(), Box<dyn Error>> {
    let mut accumulated_content = String::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4")
        .max_tokens(512u32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into()])
        .build()?;

    let mut stream = openai_client.chat().create_stream(request).await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                for chat_choice in response.choices.iter() {
                    if let Some(ref content) = chat_choice.delta.content {
                        accumulated_content.push_str(content);

                        if let Err(e) = tx.send(content.to_string()).await {
                            eprintln!("Failed to send content: {}", e);
                        }
                    }
                }
            }
            Err(err) => {
                if let Err(e) = tx.send(format!("error: {err}")).await {
                    eprintln!("Failed to send error message: {}", e);
                }
            }
        }
    }

    let chat_record = ChatRecord {
        userid,
        chatid,
        prompt: prompt.to_string(),
        response: accumulated_content.clone(),
    };

    let mut chats = chat_collection.lock().unwrap();
    chats.insert(chatid.to_string(), chat_record);

    Ok(())
}
