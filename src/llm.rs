//! The chat model: DeepSeek's OpenAI-compatible endpoint, streamed into the
//! conversation as it answers.

use std::time::Instant;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
};
use futures_util::StreamExt;
use nsh_core::prelude::*;

use crate::store::{self, Author, Message};

/// DeepSeek's OpenAI-compatible base URL: their docs' `base_url (OpenAI)`.
const BASE_URL: &str = "https://api.deepseek.com";

/// The model DeepSeek's docs name for the chat API today.
pub(crate) const MODEL: &str = "deepseek-flash";

/// The environment variable the API key comes from.
const API_KEY: &str = "DEEPSEEK_API_KEY";

/// Streams the model's answer into reply `reply_id` of `session_id`, from the
/// conversation `history` (which ends with the message just sent).
///
/// DeepSeek serves OpenAI's `/chat/completions` (their OpenAI example calls
/// `client.chat.completions.create`), not `/responses`, so this is a chat
/// completion with `stream: true` rather than a response request.
pub(crate) fn reply(session_id: u32, reply_id: u32, history: Vec<Message>) {
    let Ok(key) = std::env::var(API_KEY) else {
        store::fail_reply(session_id, reply_id, &format!("未设置环境变量 {API_KEY}"));
        return;
    };

    let messages = match messages(&history) {
        Ok(messages) => messages,
        Err(error) => {
            store::fail_reply(session_id, reply_id, &error);
            return;
        }
    };

    let request = match CreateChatCompletionRequestArgs::default()
        .model(MODEL)
        .messages(messages)
        .build()
    {
        Ok(request) => request,
        Err(error) => {
            store::fail_reply(session_id, reply_id, &error.to_string());
            return;
        }
    };

    // Under no owner, so switching sessions does not cut the answer off: it
    // keeps streaming into the store of the session that asked for it.
    Owner::detached().run(|| {
        spawn(answer(session_id, reply_id, key, request));
    });
}

/// `history` as the API's messages: who spoke, and what they said. The
/// conversation starts at its first user turn, as the API expects.
fn messages(history: &[Message]) -> Result<Vec<ChatCompletionRequestMessage>, String> {
    history
        .iter()
        .skip_while(|message| message.author != Author::User)
        .map(|message| {
            let body = message.body.to_string();
            let built = match message.author {
                Author::User => ChatCompletionRequestUserMessageArgs::default()
                    .content(body)
                    .build()
                    .map(Into::into),
                Author::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(body)
                    .build()
                    .map(Into::into),
            };
            built.map_err(|error| error.to_string())
        })
        .collect()
}

/// Reads the streamed answer into the reply, then closes it with how long it
/// took; an error takes the answer's place.
async fn answer(session_id: u32, reply_id: u32, key: String, request: CreateChatCompletionRequest) {
    let config = OpenAIConfig::new()
        .with_api_base(BASE_URL)
        .with_api_key(key);
    let client = Client::with_config(config);

    let mut stream = match client.chat().create_stream(request).await {
        Ok(stream) => stream,
        Err(error) => {
            store::fail_reply(session_id, reply_id, &error.to_string());
            return;
        }
    };

    let started = Instant::now();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                for choice in chunk.choices {
                    if let Some(text) = choice.delta.content {
                        store::append_reply(session_id, reply_id, &text);
                    }
                }
            }
            Err(error) => {
                store::fail_reply(session_id, reply_id, &error.to_string());
                return;
            }
        }
    }

    let seconds = started.elapsed().as_secs_f32();
    store::finish_reply(session_id, reply_id, MODEL, seconds);
}
