//! The responses a provider streams: OpenAI's Responses API, as DeepSeek and
//! other compatible providers serve it, at `POST /responses` with `stream:
//! true`, read as semantic server-sent events.

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::responses::{
        CreateResponse, CreateResponseArgs, EasyInputContent, EasyInputMessage, InputItem,
        InputParam, MessageType, Reasoning, ReasoningEffort, ResponseStreamEvent, Role as ApiRole,
    },
};
use futures_util::StreamExt;
use nsh_core::prelude::*;

/// How hard a reply's model thinks before it answers (`reasoning.effort`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effort {
    /// The provider's own default: the request asks for no effort.
    Default,
    /// A little.
    Low,
    /// Some: more than the low effort, less than the high one.
    Medium,
    /// As much as the provider offers.
    High,
}

/// Who said a turn of the conversation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

/// One turn of the conversation the model reads.
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub role: Role,
    pub text: SharedString,
}

/// Where a provider's requests go, and the key they carry.
#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub base_url: String,
    pub api_key: String,
}

/// What arrives of a reply, as it arrives.
#[derive(Clone, Debug, PartialEq)]
pub enum ReplyEvent {
    /// The chain of thought, as it arrived.
    Thinking(SharedString),
    /// The answer, as it arrived.
    Answer(SharedString),
    /// The last of it is in: the reply is complete.
    Done,
    /// It failed instead: this is what to show.
    Failed(SharedString),
}

/// Streams `model`'s answer to `turns`, reporting what arrives to `report`.
///
/// Under no owner, so the answer belongs to the conversation that asked for it
/// rather than to a pane that may go away.
pub fn reply(
    endpoint: Endpoint,
    model: SharedString,
    effort: Effort,
    turns: Vec<Turn>,
    report: impl Fn(ReplyEvent) + 'static,
) {
    let mut args = CreateResponseArgs::default();
    args.model(model.as_str());
    args.input(InputParam::Items(input(&turns)));
    // The provider thinks without being asked; the app asks for the effort its
    // picker names, and for none while that is the provider's default.
    if let Some(effort) = reasoning_effort(effort) {
        args.reasoning(Reasoning {
            effort: Some(effort),
            ..Reasoning::default()
        });
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

/// What `effort` asks the API for: no effort at all for the provider's own
/// default.
fn reasoning_effort(effort: Effort) -> Option<ReasoningEffort> {
    match effort {
        Effort::Default => None,
        Effort::Low => Some(ReasoningEffort::Low),
        Effort::Medium => Some(ReasoningEffort::Medium),
        Effort::High => Some(ReasoningEffort::High),
    }
}

/// `turns` as the API's input items: who spoke, and what they said.
fn input(turns: &[Turn]) -> Vec<InputItem> {
    turns
        .iter()
        .map(|turn| {
            InputItem::EasyMessage(EasyInputMessage {
                r#type: MessageType::Message,
                role: match turn.role {
                    Role::User => ApiRole::User,
                    Role::Assistant => ApiRole::Assistant,
                },
                content: EasyInputContent::Text(turn.text.to_string()),
                phase: None,
            })
        })
        .collect()
}

/// Reads the streamed answer, reporting each piece to `report` as it arrives.
async fn stream(endpoint: Endpoint, request: CreateResponse, report: impl Fn(ReplyEvent)) {
    let config = OpenAIConfig::new()
        .with_api_base(endpoint.base_url)
        .with_api_key(endpoint.api_key);
    let client = Client::with_config(config);

    let mut events = match client.responses().create_stream(request).await {
        Ok(events) => events,
        Err(error) => {
            report(ReplyEvent::Failed(error.to_string().into()));
            return;
        }
    };

    while let Some(event) = events.next().await {
        match event {
            // The chain of thought and the answer are streamed apart: each is
            // reported as its own part.
            Ok(ResponseStreamEvent::ResponseReasoningTextDelta(delta)) => {
                report(ReplyEvent::Thinking(delta.delta.into()));
            }
            Ok(ResponseStreamEvent::ResponseOutputTextDelta(delta)) => {
                report(ReplyEvent::Answer(delta.delta.into()));
            }
            Ok(ResponseStreamEvent::ResponseFailed(failed)) => {
                let message = failed
                    .response
                    .error
                    .map(|error| error.message)
                    .unwrap_or_else(|| "Response failed".to_owned());
                report(ReplyEvent::Failed(message.into()));
                return;
            }
            Ok(_) => {}
            Err(error) => {
                report(ReplyEvent::Failed(error.to_string().into()));
                return;
            }
        }
    }

    report(ReplyEvent::Done);
}
