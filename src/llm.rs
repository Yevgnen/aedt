//! The chat model: DeepSeek's Responses API, streamed into the conversation as
//! it answers.

use std::time::Instant;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::responses::{
        CreateResponse, CreateResponseArgs, EasyInputContent, EasyInputMessage, InputItem,
        InputParam, MessageType, ResponseStreamEvent, Role,
    },
};
use futures_util::StreamExt;
use nsh_core::prelude::*;

use crate::store::{self, Author, Message};

/// DeepSeek's base URL: their docs' `base_url (OpenAI)`.
const BASE_URL: &str = "https://api.deepseek.com";

/// The model DeepSeek's docs name for the Responses API today.
pub(crate) const MODEL: &str = "deepseek-flash";

/// The environment variable the API key comes from.
const API_KEY: &str = "DEEPSEEK_API_KEY";

/// Streams the model's answer into reply `reply_id` of `session_id`, from the
/// conversation `history` (which ends with the message just sent).
///
/// DeepSeek serves OpenAI's Responses API at `POST /responses` and streams it
/// as semantic server-sent events, so this asks for a response with `stream:
/// true` rather than a chat completion.
pub(crate) fn reply(session_id: u32, reply_id: u32, history: Vec<Message>) {
    let Ok(key) = std::env::var(API_KEY) else {
        store::fail_reply(session_id, reply_id, &format!("未设置环境变量 {API_KEY}"));
        return;
    };

    let request = match CreateResponseArgs::default()
        .model(MODEL)
        .input(InputParam::Items(input(&history)))
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

/// `history` as the API's input items: who spoke, and what they said. The
/// conversation starts at its first user turn, as the API expects.
fn input(history: &[Message]) -> Vec<InputItem> {
    history
        .iter()
        .skip_while(|message| message.author != Author::User)
        .map(|message| {
            InputItem::EasyMessage(EasyInputMessage {
                r#type: MessageType::Message,
                role: match message.author {
                    Author::User => Role::User,
                    Author::Assistant => Role::Assistant,
                },
                content: EasyInputContent::Text(message.body.to_string()),
                phase: None,
            })
        })
        .collect()
}

/// Reads the streamed answer into the reply, then closes it with how long it
/// took; a failure takes the answer's place.
async fn answer(session_id: u32, reply_id: u32, key: String, request: CreateResponse) {
    let config = OpenAIConfig::new()
        .with_api_base(BASE_URL)
        .with_api_key(key);
    let client = Client::with_config(config);

    let mut stream = match client.responses().create_stream(request).await {
        Ok(stream) => stream,
        Err(error) => {
            store::fail_reply(session_id, reply_id, &error.to_string());
            return;
        }
    };

    let started = Instant::now();
    while let Some(event) = stream.next().await {
        match event {
            // The visible answer: the chain of thought arrives as its own
            // `reasoning_text` deltas, which are left out.
            Ok(ResponseStreamEvent::ResponseOutputTextDelta(delta)) => {
                store::append_reply(session_id, reply_id, &delta.delta);
            }
            Ok(ResponseStreamEvent::ResponseFailed(failed)) => {
                let message = failed
                    .response
                    .error
                    .map(|error| error.message)
                    .unwrap_or_else(|| "响应失败".to_owned());
                store::fail_reply(session_id, reply_id, &message);
                return;
            }
            Ok(_) => {}
            Err(error) => {
                store::fail_reply(session_id, reply_id, &error.to_string());
                return;
            }
        }
    }

    let seconds = started.elapsed().as_secs_f32();
    store::finish_reply(session_id, reply_id, MODEL, seconds);
}
