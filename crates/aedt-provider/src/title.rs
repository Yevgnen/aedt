//! A conversation's title: what the reader asked about, in a line, written by
//! DeepSeek's flash model.

use nsh_core::{platform::HttpRequest, prelude::*};
use serde::{Deserialize, Serialize};

use crate::DEEPSEEK_MODEL;
use crate::models::failure;

/// The longest title the app keeps, in characters: a model told to be brief may
/// still run on.
const TITLE_MAX: usize = 80;

/// What the model is told to write.
const INSTRUCTIONS: &str = "You name conversations. Write a title of at most six words, in \
     the language of the conversation, with no quotes and no period at the end. Answer with the \
     title only.";

/// A title for the conversation `asked`, by the model at `base_url` with
/// `api_key`.
///
/// The chat completions API carries it, whatever API the settings chose for
/// replies: a title needs neither tools nor thinking, and every
/// OpenAI-compatible base answers this path.
pub async fn write(
    base_url: &str,
    api_key: &str,
    asked: &str,
) -> Result<SharedString, SharedString> {
    let url = format!("{}/chat/completions", base_url.trim().trim_end_matches('/'));
    let body = serde_json::to_string(&Request {
        // DeepSeek's own name for V4.1 Flash, which is the model the app's
        // replies default to as well: it refuses `deepseek-v4.1-flash` ("the
        // supported API model names are deepseek-flash, deepseek-v4-pro").
        model: DEEPSEEK_MODEL,
        messages: vec![
            RequestMessage {
                role: "system",
                content: INSTRUCTIONS,
            },
            RequestMessage {
                role: "user",
                content: asked,
            },
        ],
        stream: false,
    })
    .map_err(|error| SharedString::from(format!("Couldn't write the title request: {error}")))?;
    let request = HttpRequest::post(url, body)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json");
    let response = fetch(request).await.map_err(SharedString::from)?;
    let body = response.text();
    if !response.ok() {
        return Err(failure(response.status(), &body).into());
    }

    let answer: Answer = serde_json::from_str(&body)
        .map_err(|error| SharedString::from(format!("Couldn't read the title: {error}")))?;
    let written = answer
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .unwrap_or_default();
    let title = one_line(&written);
    if title.is_empty() {
        return Err("the model named the conversation with nothing".into());
    }

    Ok(title.into())
}

/// What the model answered, as a title: its first line, without the quotes or
/// the closing period a model may add, cut to `TITLE_MAX`.
fn one_line(written: &str) -> String {
    let line = written
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default();
    let line = line.trim_start_matches(['"', '\u{201c}', '\'']);
    let line = line.trim_end_matches(['"', '\u{201d}', '\'', '.', '\u{3002}']);

    line.trim().chars().take(TITLE_MAX).collect()
}

/// The body a title request carries.
#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    messages: Vec<RequestMessage<'a>>,
    stream: bool,
}

/// One message of that body.
#[derive(Serialize)]
struct RequestMessage<'a> {
    role: &'a str,
    content: &'a str,
}

/// A model's answer, of what the app reads.
#[derive(Deserialize)]
struct Answer {
    choices: Vec<Choice>,
}

/// One answer of that list.
#[derive(Deserialize)]
struct Choice {
    message: AnswerMessage,
}

/// What one answer says.
#[derive(Deserialize)]
struct AnswerMessage {
    content: Option<String>,
}
