//! The replies a provider streams from its chat completions: OpenAI's
//! `/chat/completions`, as DeepSeek and the gateways that serve it do, read as
//! server-sent events.

use std::sync::LazyLock;

use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
    CreateChatCompletionRequestArgs, ReasoningEffort,
};
use futures_util::StreamExt;
use nsh_core::prelude::*;
use reqwest::{Client, StatusCode, header::CONTENT_TYPE};
use serde::Deserialize;

use crate::responses::{Effort, Endpoint, ReplyEvent, Role, Turn};

/// Streams `model`'s answer to `turns`, reporting what arrives to `report`.
///
/// The chain of thought streams beside the answer, under the name each
/// provider's API gives it (`reasoning_content`, or `reasoning`): both parts
/// are reported, as the Responses API's are.
pub fn reply(
    endpoint: Endpoint,
    model: SharedString,
    effort: Effort,
    turns: Vec<Turn>,
    report: impl Fn(ReplyEvent) + 'static,
) {
    let mut args = CreateChatCompletionRequestArgs::default();
    args.model(model.as_str());
    args.messages(messages(&turns));
    args.stream(true);
    // The provider thinks without being asked; the app asks for the effort its
    // picker names, and for none while that is the provider's default.
    if let Some(effort) = reasoning_effort(effort) {
        args.reasoning_effort(effort);
    }
    let request = match args.build() {
        Ok(request) => request,
        Err(error) => {
            report(ReplyEvent::Failed(error.to_string().into()));
            return;
        }
    };

    Owner::detached().run(move || {
        spawn(stream(endpoint, request, report));
    });
}

/// What `effort` asks the API for: none for the provider's own default.
fn reasoning_effort(effort: Effort) -> Option<ReasoningEffort> {
    match effort {
        Effort::Default => None,
        Effort::Low => Some(ReasoningEffort::Low),
        Effort::Medium => Some(ReasoningEffort::Medium),
        Effort::High => Some(ReasoningEffort::High),
    }
}

/// `turns` as the API's messages: who spoke, and what they said.
fn messages(turns: &[Turn]) -> Vec<ChatCompletionRequestMessage> {
    turns
        .iter()
        .filter_map(|turn| {
            let text = turn.text.to_string();
            match turn.role {
                Role::User => ChatCompletionRequestUserMessageArgs::default()
                    .content(text)
                    .build()
                    .ok()
                    .map(ChatCompletionRequestMessage::User),
                Role::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(text)
                    .build()
                    .ok()
                    .map(ChatCompletionRequestMessage::Assistant),
            }
        })
        .collect()
}

/// Reads the streamed answer, reporting each piece to `report` as it arrives.
async fn stream(
    endpoint: Endpoint,
    request: CreateChatCompletionRequest,
    report: impl Fn(ReplyEvent),
) {
    let client = match client() {
        Ok(client) => client,
        Err(why) => {
            report(ReplyEvent::Failed(why.into()));
            return;
        }
    };
    let url = format!(
        "{}/chat/completions",
        endpoint.base_url.trim_end_matches('/')
    );
    let body = match serde_json::to_vec(&request) {
        Ok(body) => body,
        Err(error) => {
            report(ReplyEvent::Failed(
                format!("failed to encode the request: {error}").into(),
            ));
            return;
        }
    };

    let sent = client
        .post(url)
        .bearer_auth(&endpoint.api_key)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await;
    let response = match sent {
        Ok(response) => response,
        Err(error) => {
            report(ReplyEvent::Failed(
                format!("failed to send the request: {error}").into(),
            ));
            return;
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        report(ReplyEvent::Failed(refusal(status, &body).into()));
        return;
    }

    let mut chunks = response.bytes_stream();
    // The bytes of a line not yet ended, and of an event's `data:` lines not
    // yet ended by an empty one.
    let mut buffer = String::new();
    let mut data = String::new();
    let mut done = false;
    while let Some(chunk) = chunks.next().await {
        let bytes = match chunk {
            Ok(bytes) => bytes,
            Err(error) => {
                report(ReplyEvent::Failed(
                    format!("failed to read the reply: {error}").into(),
                ));
                return;
            }
        };

        buffer.push_str(&String::from_utf8_lossy(&bytes));
        while let Some(at) = buffer.find('\n') {
            let line = buffer.drain(..=at).collect::<String>();
            let line = line.trim_end_matches(['\n', '\r']);
            if read_line(line, &mut data, &report, &mut done) {
                return;
            }
        }
    }

    // The stream ended: the last line, and an event it never ended with an
    // empty line, still hold what came.
    if read_line(&buffer, &mut data, &report, &mut done) {
        return;
    }
    if !data.is_empty() && emit(&data, &report, &mut done) {
        return;
    }
    if !done {
        report(ReplyEvent::Done);
    }
}

/// Reads one line of the stream: a `data:` line adds to the event, and an
/// empty one ends it. Whether the reply is over.
fn read_line(line: &str, data: &mut String, report: &impl Fn(ReplyEvent), done: &mut bool) -> bool {
    if line.is_empty() {
        if data.is_empty() {
            return *done;
        }
        let event = std::mem::take(data);
        return emit(&event, report, done);
    }
    let Some(rest) = line.strip_prefix("data:") else {
        // `event:`, `id:` and `: comment` lines carry nothing the reply shows.
        return *done;
    };
    if !data.is_empty() {
        data.push('\n');
    }
    data.push_str(rest.trim_start());
    false
}

/// Reads one event's `data`: its pieces of the reply, and its end. Whether the
/// reply is over.
fn emit(data: &str, report: &impl Fn(ReplyEvent), done: &mut bool) -> bool {
    if data.trim() == "[DONE]" {
        *done = true;
        report(ReplyEvent::Done);
        return true;
    }

    let chunk = match serde_json::from_str::<Chunk>(data) {
        Ok(chunk) => chunk,
        Err(error) => {
            *done = true;
            report(ReplyEvent::Failed(
                format!("failed to read the reply: {error}").into(),
            ));
            return true;
        }
    };
    if let Some(error) = chunk.error {
        *done = true;
        report(ReplyEvent::Failed(error.message().into()));
        return true;
    }

    for choice in chunk.choices {
        if let Some(thought) = choice.delta.thought() {
            report(ReplyEvent::Thinking(thought.to_owned().into()));
        }
        if let Some(content) = choice.delta.content.filter(|content| !content.is_empty()) {
            report(ReplyEvent::Answer(content.into()));
        }
    }

    false
}

/// Why the provider refused the reply: the message it gave, else its body.
fn refusal(status: StatusCode, body: &str) -> String {
    let why = serde_json::from_str::<ErrorBody>(body)
        .ok()
        .and_then(|body| body.error.message)
        .filter(|message| !message.is_empty());
    match why {
        Some(why) => format!("the provider refused the reply ({status}): {why}"),
        None => format!("the provider refused the reply ({status}): {}", body.trim()),
    }
}

/// The HTTP client every reply streams through: its connections are reused.
fn client() -> Result<&'static Client, String> {
    static CLIENT: LazyLock<Result<Client, String>> = LazyLock::new(|| {
        Client::builder()
            .build()
            .map_err(|error| format!("failed to build the HTTP client: {error}"))
    });
    match &*CLIENT {
        Ok(client) => Ok(client),
        Err(why) => Err(why.clone()),
    }
}

/// One piece of the streamed reply, as the provider wrote it.
#[derive(Deserialize)]
struct Chunk {
    #[serde(default)]
    choices: Vec<Choice>,
    /// The failure the provider streamed in place of an answer.
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Delta,
}

/// What one piece of a reply carries.
#[derive(Default, Deserialize)]
struct Delta {
    /// The answer, as it arrived.
    #[serde(default)]
    content: Option<String>,
    /// The chain of thought, as DeepSeek and LithosAI name it.
    #[serde(default)]
    reasoning_content: Option<String>,
    /// The chain of thought, as OpenRouter and others name it.
    #[serde(default)]
    reasoning: Option<String>,
}

impl Delta {
    /// The chain of thought this piece carries, under either name.
    fn thought(&self) -> Option<&str> {
        self.reasoning_content
            .as_deref()
            .or(self.reasoning.as_deref())
            .filter(|thought| !thought.is_empty())
    }
}

/// A failure, as the provider's API words it.
#[derive(Deserialize)]
struct ApiError {
    #[serde(default)]
    message: Option<String>,
}

impl ApiError {
    /// What to show of it.
    fn message(self) -> String {
        self.message
            .unwrap_or_else(|| "the provider failed the reply".to_owned())
    }
}

/// The body a provider refuses a request with.
#[derive(Deserialize)]
struct ErrorBody {
    error: ApiError,
}
