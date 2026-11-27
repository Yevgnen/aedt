//! A session's title: the conversation named by DeepSeek's flash model, its
//! words typing into the header and the sidebar as they come.

use std::time::Duration;

use aedt_provider::title;
use aedt_settings as settings;
use aedt_store as store;
use nsh_core::prelude::*;

/// How long one character of a title takes to type in.
const TYPE: Duration = Duration::from_millis(24);

/// How much of a conversation its title's model reads.
const READ: usize = 2_000;

/// Whether a title can be written at all: a DeepSeek provider with a key is set
/// up (tracked where it is read).
pub fn available() -> bool {
    settings::deepseek().is_some()
}

/// Names `session` from its conversation, with DeepSeek's flash model, and types
/// the title in as it comes: nothing while no DeepSeek provider is set up, or
/// while a title is already being written.
pub fn write(session_id: u32) {
    let Some(deepseek) = settings::deepseek() else {
        return;
    };
    if store::title_writing(session_id) {
        return;
    }
    let read = conversation(session_id);
    if read.trim().is_empty() {
        return;
    }
    store::title_started(session_id);

    // Under no owner of its own: the pane going away must not stop the title
    // from coming into the store.
    Owner::detached().run(move || {
        spawn(async move {
            match title::write(&deepseek.base_url, &deepseek.api_key, &read).await {
                Ok(title) => type_in(session_id, title.as_str()).await,
                Err(why) => {
                    eprintln!("title: {why}");
                    store::title_failed(session_id);
                }
            }
        })
    });
}

/// Types `title` into `session`, a character at a time, and keeps it whole once
/// the last one is in. A reader who asked for less motion sees it at once.
async fn type_in(session_id: u32, title: &str) {
    if reduce_motion() {
        if store::title_typed(session_id, title) {
            store::title_finished(session_id, title);
        } else {
            store::title_failed(session_id);
        }

        return;
    }

    let mut typed = String::new();
    for character in title.chars() {
        typed.push(character);
        if !store::title_typed(session_id, &typed) {
            store::title_failed(session_id);

            return;
        }
        timer(TYPE).await;
    }
    store::title_finished(session_id, title);
}

/// What the model reads of `session`: its messages' words, up to `READ`.
fn conversation(session_id: u32) -> String {
    let mut read = String::new();
    for index in 0..store::message_count(session_id) {
        let Some(message) = store::message_at(session_id, index) else {
            continue;
        };
        let body = message.body();
        let body = body.trim();
        if body.is_empty() {
            continue;
        }
        if !read.is_empty() {
            read.push_str("\n\n");
        }
        read.push_str(body);
        if read.chars().count() >= READ {
            break;
        }
    }

    read.chars().take(READ).collect()
}
