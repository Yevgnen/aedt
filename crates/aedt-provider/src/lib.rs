//! An OpenAI-compatible provider: the models it lists, and the effort a reply
//! asks its model for.

pub mod models;
pub mod speech;
pub mod title;

use serde::{Deserialize, Serialize};

/// How hard a reply's model thinks before it answers (`reasoning.effort`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// DeepSeek's base URL: their docs' `base_url (OpenAI)`.
pub const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";

/// The model DeepSeek's docs name for the Responses API today.
pub const DEEPSEEK_MODEL: &str = "deepseek-flash";

/// The model a session's title is written by: DeepSeek's own V4.1 Flash, small
/// enough that naming a conversation costs little.
pub const DEEPSEEK_TITLE_MODEL: &str = "deepseek-v4.1-flash";
