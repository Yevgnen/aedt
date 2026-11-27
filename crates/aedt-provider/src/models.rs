//! What a provider lists: its models, read from its `/models` answer.

use nsh_core::{platform::HttpRequest, prelude::*};
use serde::Deserialize;

/// One model of a provider's `/models` answer, of what the app reads.
#[derive(Clone, Debug, PartialEq)]
pub struct ListedModel {
    /// Its id, as the provider names it: what a request asks for.
    pub id: String,
    /// The provider's own name for it (DeepSeek gives one, `OpenAI` does not).
    pub name: Option<String>,
}

/// The models `base_url` lists, by id, in order. It reads the answer's `data`
/// and each model's `id` only, so whatever else a provider adds or leaves out
/// of a model does not fail the whole list.
pub async fn list_models(base_url: &str, api_key: &str) -> Result<Vec<ListedModel>, SharedString> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let request = HttpRequest::get(url).header("Authorization", format!("Bearer {api_key}"));
    let response = fetch(request).await.map_err(SharedString::from)?;
    let body = response.text();
    if !response.ok() {
        return Err(failure(response.status(), &body).into());
    }

    let listed: Listed = serde_json::from_str(&body)
        .map_err(|error| SharedString::from(format!("Couldn't read the model list: {error}")))?;
    let mut models = listed
        .data
        .into_iter()
        .map(|model| ListedModel {
            id: model.id,
            name: model.name,
        })
        .collect::<Vec<_>>();
    models.sort_by(|a, b| a.id.cmp(&b.id));
    models.dedup_by(|a, b| a.id == b.id);
    Ok(models)
}

/// What a refused answer says: its status, and the API's own message when the
/// body carries one.
pub(crate) fn failure(status: u16, body: &str) -> String {
    let message = serde_json::from_str::<Failed>(body)
        .ok()
        .and_then(|failed| failed.error)
        .map(|error| error.message);
    match message {
        Some(message) if !message.is_empty() => format!("{status} {message}"),
        _ => {
            let text: String = body.trim().chars().take(200).collect();
            if text.is_empty() {
                status.to_string()
            } else {
                format!("{status} {text}")
            }
        }
    }
}

/// A provider's `/models` answer, of what it says.
#[derive(Deserialize)]
struct Listed {
    data: Vec<ListedWire>,
}

/// One model of that answer, of what the app reads.
#[derive(Deserialize)]
struct ListedWire {
    id: String,
    /// The provider's own name for it.
    name: Option<String>,
}

/// What an answer that failed says.
#[derive(Deserialize)]
struct Failed {
    error: Option<FailedError>,
}

/// A failure's own words.
#[derive(Deserialize)]
struct FailedError {
    message: String,
}
