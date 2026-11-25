//! The chat model: DeepSeek's Responses API, streamed into the conversation as
//! it answers.

use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::responses::{
        CreateResponse, CreateResponseArgs, EasyInputContent, EasyInputMessage, InputItem,
        InputParam, MessageType, Reasoning, ReasoningEffort, ResponseStreamEvent, Role,
    },
};
use futures_util::StreamExt;
use nsh_core::prelude::*;

use crate::{
    settings::Choice,
    store::{self, Author, Message},
};

/// DeepSeek's base URL: their docs' `base_url (OpenAI)`.
pub(crate) const BASE_URL: &str = "https://api.deepseek.com";

/// The model DeepSeek's docs name for the Responses API today.
pub(crate) const MODEL: &str = "deepseek-flash";

/// The environment variable the API key comes from.
pub(crate) const API_KEY: &str = "DEEPSEEK_API_KEY";

/// Streams `model`'s answer into reply `reply_id` of `session_id`, from the
/// conversation `history` (which ends with the message just sent).
///
/// DeepSeek serves OpenAI's Responses API at `POST /responses` and streams it
/// as semantic server-sent events, so this asks for a response with `stream:
/// true` rather than a chat completion.
pub(crate) fn reply(session_id: u32, reply_id: u32, history: Vec<Message>, model: &Choice) {
    let Ok(key) = std::env::var(API_KEY) else {
        store::fail_reply(session_id, reply_id, &format!("未设置环境变量 {API_KEY}"));
        return;
    };

    let request = match CreateResponseArgs::default()
        .model(model.id.as_str())
        .input(InputParam::Items(input(&history)))
        // DeepSeek thinks by default; the app asks for the high effort its API
        // names (`reasoning.effort: "high"`).
        .reasoning(Reasoning {
            effort: Some(ReasoningEffort::High),
            ..Reasoning::default()
        })
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
    let reveal = Rc::new(RefCell::new(Reveal::default()));
    let name = model.label.clone();
    Owner::detached().run(move || {
        spawn(answer(key, request, reveal.clone()));
        spawn(reveal_answer(session_id, reply_id, reveal, name));
    });
}

/// What has arrived of an answer but is not shown yet. The network task fills
/// it; the reveal task drains it at a steady pace, so the text flows instead of
/// landing in token-sized bursts.
#[derive(Default)]
struct Reveal {
    /// The chain of thought, as it arrived.
    thinking: String,
    /// The answer, as it arrived.
    answer: String,
    /// The stream ended (with nothing left to wait for).
    done: bool,
    /// It failed instead: this replaces the answer.
    failed: Option<String>,
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

/// Reads the streamed answer into `reveal`, which the reveal task shows at its
/// own pace; the reply is closed there once the last of it is out.
async fn answer(key: String, request: CreateResponse, reveal: Rc<RefCell<Reveal>>) {
    let config = OpenAIConfig::new()
        .with_api_base(BASE_URL)
        .with_api_key(key);
    let client = Client::with_config(config);

    let mut stream = match client.responses().create_stream(request).await {
        Ok(stream) => stream,
        Err(error) => {
            reveal.borrow_mut().failed = Some(error.to_string());
            return;
        }
    };

    while let Some(event) = stream.next().await {
        match event {
            // The chain of thought and the answer are streamed apart: each is
            // read into the reply's own buffer, in its own place.
            Ok(ResponseStreamEvent::ResponseReasoningTextDelta(delta)) => {
                reveal.borrow_mut().thinking.push_str(&delta.delta);
            }
            Ok(ResponseStreamEvent::ResponseOutputTextDelta(delta)) => {
                reveal.borrow_mut().answer.push_str(&delta.delta);
            }
            Ok(ResponseStreamEvent::ResponseFailed(failed)) => {
                let message = failed
                    .response
                    .error
                    .map(|error| error.message)
                    .unwrap_or_else(|| "响应失败".to_owned());
                reveal.borrow_mut().failed = Some(message);
                return;
            }
            Ok(_) => {}
            Err(error) => {
                reveal.borrow_mut().failed = Some(error.to_string());
                return;
            }
        }
    }

    reveal.borrow_mut().done = true;
}

/// How often the reveal task looks at what arrived: about one frame of a 30 Hz
/// display, quick enough to read as typing and slow enough to keep the writes
/// (each one re-renders the reply) to thirty a second.
const REVEAL_TICK: Duration = Duration::from_millis(33);

/// The most characters one tick shows while the answer is still arriving: the
/// pace rises with the backlog and falls back to a couple of characters, so it
/// keeps up with the model without bursting.
const REVEAL_STEP: usize = 40;

/// Shows what `answer` put in `reveal`, a few characters a tick, then closes the
/// reply with how long it took; a failure takes the answer's place.
async fn reveal_answer(
    session_id: u32,
    reply_id: u32,
    reveal: Rc<RefCell<Reveal>>,
    name: SharedString,
) {
    let started = Instant::now();

    loop {
        let (thinking, answer, remaining, done, failed) = {
            let mut reveal = reveal.borrow_mut();
            let done = reveal.done;
            let thinking = take_ready(&mut reveal.thinking, done);
            let answer = take_ready(&mut reveal.answer, done);
            (
                thinking,
                answer,
                reveal.thinking.chars().count() + reveal.answer.chars().count(),
                done,
                reveal.failed.take(),
            )
        };

        if thinking.is_some() || answer.is_some() {
            store::append_reply(
                session_id,
                reply_id,
                thinking.as_deref().unwrap_or_default(),
                answer.as_deref().unwrap_or_default(),
            );
        }
        if let Some(error) = failed {
            store::fail_reply(session_id, reply_id, &error);
            return;
        }
        if done && remaining == 0 {
            let seconds = started.elapsed().as_secs_f32();
            store::finish_reply(session_id, reply_id, name.as_str(), seconds);
            return;
        }

        timer(REVEAL_TICK).await;
    }
}

/// What of one buffer a tick shows: everything once the stream is over, else a
/// share of the backlog (at least two characters, at most `REVEAL_STEP`).
fn take_ready(pending: &mut String, done: bool) -> Option<String> {
    let count = pending.chars().count();
    if count == 0 {
        return None;
    }
    let step = if done {
        count
    } else {
        (count / 8).clamp(2, REVEAL_STEP).min(count)
    };
    Some(take_chars(pending, step))
}

/// Up to `max` characters off the front of `text`, whole ones only.
fn take_chars(text: &mut String, max: usize) -> String {
    let end = text
        .char_indices()
        .nth(max)
        .map_or(text.len(), |(at, _)| at);
    let taken = text[..end].to_owned();
    text.drain(..end);
    taken
}
