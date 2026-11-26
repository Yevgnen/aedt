//! The replies a provider streams from its chat completions: OpenAI's
//! `/chat/completions`, as DeepSeek and the gateways that serve it do, read as
//! server-sent events.

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs, ReasoningEffort,
    },
};
use futures_util::StreamExt;
use nsh_core::prelude::*;

use crate::responses::{Effort, Endpoint, ReplyEvent, Role, Turn};

/// Streams `model`'s answer to `turns`, reporting what arrives to `report`.
///
/// The chat completions carry the answer alone: a provider that thinks first
/// does not stream that part here, so the reply has no thinking to show.
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
    let config = OpenAIConfig::new()
        .with_api_base(endpoint.base_url)
        .with_api_key(endpoint.api_key);
    let client = Client::with_config(config);

    let mut events = match client.chat().create_stream(request).await {
        Ok(events) => events,
        Err(error) => {
            report(ReplyEvent::Failed(error.to_string().into()));
            return;
        }
    };

    while let Some(event) = events.next().await {
        match event {
            Ok(response) => {
                for choice in response.choices {
                    if let Some(content) = choice.delta.content
                        && !content.is_empty()
                    {
                        report(ReplyEvent::Answer(content.into()));
                    }
                }
            }
            Err(error) => {
                report(ReplyEvent::Failed(error.to_string().into()));
                return;
            }
        }
    }

    report(ReplyEvent::Done);
}
