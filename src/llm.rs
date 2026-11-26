//! The chat: the conversation's model is asked through its provider, and the
//! answer is shown a few characters at a time as it arrives.

use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use aedt_provider::responses::{self, Endpoint, ReplyEvent, Role, Turn};
use aedt_store::{self as store, Author, Message};
use nsh_core::prelude::*;

use aedt_settings as settings;
use aedt_settings::ProviderKind;

/// Sends `text` to the conversation and starts the reply its model writes: the
/// store keeps the conversation, the settings name the model and its provider.
pub(crate) fn ask(session_id: u32, text: &str) {
    let Some(sent) = store::send(session_id, text) else {
        return;
    };
    let model = settings::selected();
    let effort = settings::effort();
    if model.base_url.trim().is_empty() || model.api_key.trim().is_empty() {
        store::fail_reply(session_id, sent.reply_id, "API URL or API key is missing");
        return;
    }

    // Under no owner, so switching sessions does not cut the answer off: it
    // keeps streaming into the store of the session that asked for it.
    let reveal = Rc::new(RefCell::new(Reveal::default()));
    Owner::detached().run(move || {
        let endpoint = Endpoint {
            base_url: model.base_url.clone(),
            api_key: model.api_key.clone(),
        };
        let collect = {
            let reveal = reveal.clone();
            move |event| collect(&reveal, event)
        };
        match model.kind {
            ProviderKind::OpenAiResponses => responses::reply(
                endpoint,
                model.id.clone(),
                effort,
                turns(&sent.history),
                collect,
            ),
            ProviderKind::OpenAiChatCompletions => aedt_provider::chat::reply(
                endpoint,
                model.id.clone(),
                effort,
                turns(&sent.history),
                collect,
            ),
        }
        spawn(reveal_answer(
            session_id,
            sent.reply_id,
            reveal.clone(),
            model.label.clone(),
        ));
    });
}

/// What arrived of an answer, kept until the reveal task shows it.
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

/// Keeps `event` in `reveal` for the reveal task to show.
fn collect(reveal: &Rc<RefCell<Reveal>>, event: ReplyEvent) {
    let mut reveal = reveal.borrow_mut();
    match event {
        ReplyEvent::Thinking(text) => reveal.thinking.push_str(text.as_str()),
        ReplyEvent::Answer(text) => reveal.answer.push_str(text.as_str()),
        ReplyEvent::Done => reveal.done = true,
        ReplyEvent::Failed(why) => reveal.failed = Some(why.to_string()),
    }
}

/// `history` as the provider's turns: the conversation starts at its first user
/// message, as the API expects.
fn turns(history: &[Message]) -> Vec<Turn> {
    history
        .iter()
        .skip_while(|message| message.author != Author::User)
        .map(|message| Turn {
            role: match message.author {
                Author::User => Role::User,
                Author::Assistant => Role::Assistant,
            },
            text: message.body.clone(),
        })
        .collect()
}

/// How often the reveal task looks at what arrived: about one frame of a 30 Hz
/// display, quick enough to read as typing and slow enough to keep the writes
/// (each one re-renders the reply) to thirty a second.
const REVEAL_TICK: Duration = Duration::from_millis(33);

/// The most characters one tick shows while the answer is still arriving: the
/// pace rises with the backlog and falls back to a couple of characters, so it
/// keeps up with the model without bursting.
const REVEAL_STEP: usize = 40;

/// Shows what the provider reported, a few characters a tick, then closes the
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
