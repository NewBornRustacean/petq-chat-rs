use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client as OpenaiClient,
};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use std:: sync::Arc;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use anyhow::{Result, Error};

#[derive(serde::Deserialize, ToSchema, IntoParams)]
pub struct ChatParams {
    prompt: String,
}


pub struct AppState {
    pub openai_client: OpenaiClient<OpenAIConfig>,
    // pub records: Arc<Mutex<RecordStore>>,
}

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
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .max_tokens(512u32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(&*params.prompt)
            .build()
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let mut stream = state.openai_client.chat().create_stream(request).await.unwrap();

    let body = axum::body::Body::from_stream(async_stream::stream! {
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for chat_choice in response.choices {
                        if let Some(content) = chat_choice.delta.content {
                            yield Ok::<std::string::String, Error>(content);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error in streaming response: {}", err);
                }
            }
        }
    });

    Response::new(body)
}

