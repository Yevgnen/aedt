//! An OpenAI-compatible provider: the models it lists, and the responses it
//! streams.

pub mod models;
pub mod responses;

/// DeepSeek's base URL: their docs' `base_url (OpenAI)`.
pub const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";

/// The model DeepSeek's docs name for the Responses API today.
pub const DEEPSEEK_MODEL: &str = "deepseek-flash";
