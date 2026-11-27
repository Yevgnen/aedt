//! What the settings keep between runs: the providers the reader configured,
//! the models they switched off, and the choice the composer holds. A
//! provider's API key is not among them: it is kept beside this file, readable
//! by this user only ([`crate::secrets`]).

use nsh_core::storage::Storage;
use serde::{Deserialize, Serialize};

use crate::{Effort, Thinking};

/// The storage key the settings are kept under.
const KEY: &str = "aedt.settings";

/// How a kept provider speaks: the two kinds the app has, and the one it had
/// before chat completions.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) enum Kind {
    #[serde(rename = "openAiResponses")]
    Responses,
    #[serde(rename = "openAiChatCompletions")]
    ChatCompletions,
    /// The single kind the app had when it spoke only the Responses API.
    #[serde(rename = "openAiCompatible")]
    Compatible,
}

/// A provider as it is kept: how to reach it, never the key that does.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Provider {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) kind: Kind,
    pub(crate) base_url: String,
    /// The models the reader switched off, by id.
    #[serde(default)]
    pub(crate) disabled: Vec<String>,
}

/// The settings as they are kept.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Kept {
    #[serde(default)]
    pub(crate) providers: Vec<Provider>,
    /// The model the next reply asked, by id.
    #[serde(default)]
    pub(crate) selected: String,
    /// The provider that served it, while the reader picked one.
    #[serde(default)]
    pub(crate) selected_provider: Option<u32>,
    /// How hard it thought; none for the app's own default.
    #[serde(default)]
    pub(crate) effort: Option<Effort>,
    /// How its thinking shows; none for the app's own default.
    #[serde(default)]
    pub(crate) thinking: Option<Thinking>,
}

/// The settings the app kept, while it kept a readable set.
pub(crate) fn load() -> Option<Kept> {
    let kept = Storage::get_item(KEY)?;
    match serde_json::from_str(&kept) {
        Ok(kept) => Some(kept),
        Err(error) => {
            eprintln!("settings: the kept settings could not be read: {error}");
            None
        }
    }
}

/// Keeps `kept`, or says why it could not.
pub(crate) fn save(kept: &Kept) {
    let kept = match serde_json::to_string(kept) {
        Ok(kept) => kept,
        Err(error) => {
            eprintln!("settings: the settings could not be written: {error}");
            return;
        }
    };
    if let Err(error) = Storage::set_item(KEY, kept) {
        eprintln!("settings: {error}");
    }
}
