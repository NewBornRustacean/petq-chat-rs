use async_openai::types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};
use async_openai::Client as OpenaiClient;
use axum::{
    extract::{Path, Query, State},
    response::sse::Event,
    response::{IntoResponse, Sse},
};
use futures::{FutureExt, StreamExt};
use mongodb::bson::uuid;
use mongodb::{Client as MongoClient, Collection};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct ChatParams {
    prompt: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRecord {
    userid: Uuid,
    chatid: Uuid,
    prompt: String,
    response: String,
}

type ChatQueue = Arc<Mutex<mpsc::Sender<ChatRecord>>>;

pub async fn chat_stream_handler(
    Path((userid, chatid)): Path<(Uuid, Uuid)>,
    Query(params): Query<ChatParams>,
    State(chat_queue): State<ChatQueue>,
) -> impl IntoResponse {
    let (tx, rx) = mpsc::channel(100);
    let chat_queue_clone = Arc::clone(&chat_queue);

    tokio::spawn(async move {
        if let Err(e) = generate_chat_stream(params.prompt.as_str(), tx, chat_queue_clone, userid, chatid).await {
            println!("failed to stream chat: {}", e);
        }
    });

    let stream = ReceiverStream::new(rx);
    Sse::new(stream.map(|msg| Ok::<_, std::convert::Infallible>(Event::default().data(msg))))
}

pub async fn generate_chat_stream(
    prompt: &str,
    tx: mpsc::Sender<String>,
    chat_queue: ChatQueue,
    userid: Uuid,
    chatid: Uuid,
) -> Result<(), Box<dyn Error>> {
    let openai_client = OpenaiClient::new();
    let mut accumulated_content = String::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .max_tokens(512u32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(&*prompt)
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

    let chat_record = ChatRecord {
        userid,
        chatid,
        prompt: prompt.to_string(),
        response: accumulated_content.clone(),
    };

    if let Err(e) = chat_queue.lock().await.send(chat_record).await {
        eprintln!("failed to push job to queue: {}", e);
    }


    Ok(())
}

pub async fn insert_to_db_from_queue( mut rx: mpsc::Receiver<ChatRecord>) -> Result<(), Box<dyn Error>> {
    // let collection: Collection<ChatRecord> = db_client.database("chat_db").collection("chat_records");

    while let Some(job) = rx.recv().await {
        println!("consumed: {:?}", job);
        // // Insert the job (input-output pair) into MongoDB
        // if let Err(e) = collection.insert_one(job).await {
        //     eprintln!("failed to insert job into MongoDB: {}", e);
        // }
    }
    Ok(())
}
