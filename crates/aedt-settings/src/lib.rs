//! The app's settings: its model providers, and the models each one serves.
//! The providers are the app's state; asking one for its models fills them
//! from its own `/models`, so nothing here is typed by hand.

use std::env;

use aedt_provider::Effort;
use aedt_provider::models::{ListedModel, list_models};
use aedt_provider::{DEEPSEEK_BASE_URL, DEEPSEEK_MODEL};
use nsh_core::prelude::*;
use serde::{Deserialize, Serialize};

mod secrets;
mod stored;

/// How a reply's thinking shows while it arrives and once it is done.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Thinking {
    /// Folded away, even while it arrives.
    Collapsed,
    /// Always open.
    Expanded,
    /// Open while it arrives, folded away once the reply goes on.
    Auto,
    /// Its last few lines, following the end as it arrives.
    #[default]
    Preview,
}

/// The blank the conversation keeps on each side, in percent of its width,
/// until the reader picks another.
pub const DEFAULT_MARGIN: u32 = 15;

/// The widest blank the conversation may keep on each side: past it the column
/// is too narrow for a message to read in.
pub const MAX_MARGIN: u32 = 35;

/// A provider's API: the shape of its requests and of its model list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    /// OpenAI's Responses API: `/responses`, streamed as semantic events.
    OpenAiResponses,
    /// OpenAI's chat completions API: `/chat/completions`, streamed as
    /// server-sent events.
    OpenAiChatCompletions,
}

impl ProviderKind {
    /// How a provider of this kind is kept.
    fn kept(self) -> stored::Kind {
        match self {
            Self::OpenAiResponses => stored::Kind::Responses,
            Self::OpenAiChatCompletions => stored::Kind::ChatCompletions,
        }
    }

    /// The kind `kept` names. The one the app had before chat completions is
    /// settled by the endpoint: DeepSeek's and Ollama's own serve the Responses
    /// API, the gateways chat completions.
    fn restored(kept: stored::Kind, base_url: &str) -> Self {
        match kept {
            stored::Kind::Responses => Self::OpenAiResponses,
            stored::Kind::ChatCompletions => Self::OpenAiChatCompletions,
            stored::Kind::Compatible => {
                let base = base_url.trim_end_matches('/');
                if base == DEEPSEEK_BASE_URL || base == "https://ollama.com/v1" {
                    Self::OpenAiResponses
                } else {
                    Self::OpenAiChatCompletions
                }
            }
        }
    }
}

/// One model a provider serves.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    /// Its id, as the provider names it: what a chat asks for.
    pub id: SharedString,
    /// What the app calls it: the provider's own name for it, or its id when
    /// the provider gives none.
    pub label: SharedString,
    /// Whether the app offers it; everything a provider lists starts offered.
    pub enabled: bool,
}

/// A provider's models, and how the last ask for them is going.
#[derive(Clone, Debug, PartialEq)]
enum Models {
    /// Not asked for yet.
    Idle,
    /// Asked for: the models it had still show while the answer comes.
    Loading(Vec<Model>),
    /// Its models, as it listed them.
    Ready(Vec<Model>),
    /// The ask failed: the models it had still show, and why it failed.
    Failed(Vec<Model>, SharedString),
}

impl Models {
    /// The models on show: what the provider last listed, or none.
    fn shown(&self) -> &[Model] {
        match self {
            Self::Idle => &[],
            Self::Loading(models) | Self::Ready(models) | Self::Failed(models, _) => models,
        }
    }
}

/// One provider: what it needs to answer, and the models it lists.
#[derive(Clone, Debug, PartialEq)]
struct Provider {
    id: u32,
    name: SharedString,
    kind: ProviderKind,
    /// Its base URL: `/models` under it lists the models.
    base_url: String,
    /// The key its requests carry.
    api_key: String,
    /// The environment variable `api_key` came from, while it came from one:
    /// what the form tells the reader, and what a reset goes back to.
    env_key: Option<SharedString>,
    /// The models the reader switched off, by id: their choice outlives the list
    /// a provider answers with.
    disabled: Vec<SharedString>,
    /// The models it lists, as its last ask found them.
    models: Models,
    /// Which ask the last answer belongs to: an older answer does not land.
    ask: u64,
    /// Whether the reader kept it: a provider being added is in the list only
    /// while its form is filled in.
    saved: bool,
}

/// One local MCP server: what starts it, and whether the agent does.
#[derive(Clone, Debug, PartialEq)]
struct Mcp {
    id: u32,
    /// What the app calls it, and the prefix of the tools it serves.
    name: SharedString,
    /// The program it runs.
    command: String,
    /// Its arguments, as they were typed.
    args: String,
    /// Its environment, a `KEY=value` a line, as it was typed.
    env: String,
    /// Whether the agent starts it, and offers the tools it lists.
    enabled: bool,
    /// Whether the reader kept it: a server being added is in the list only
    /// while its form is filled in.
    saved: bool,
}

impl Mcp {
    /// The server `kept` names.
    fn restored(kept: stored::Mcp) -> Self {
        Self {
            id: kept.id,
            name: kept.name.into(),
            command: kept.command,
            args: kept.args,
            env: kept.env,
            enabled: kept.enabled,
            saved: true,
        }
    }

    /// The server as the agent starts it: its arguments and its environment
    /// read out of the text the reader typed.
    fn choice(&self) -> McpServer {
        McpServer {
            name: self.name.trim().to_owned(),
            command: self.command.trim().to_owned(),
            args: words(&self.args),
            env: variables(&self.env),
        }
    }
}

/// The arguments `text` names: words separated by blank space, with quotes
/// keeping one together, as a shell reads them. A quote that never closes runs
/// to the end.
fn words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = None;
    let mut started = false;
    for character in text.chars() {
        match quoted {
            Some(quote) if character == quote => quoted = None,
            Some(_) => {
                started = true;
                word.push(character);
            }
            None if character == '"' || character == '\'' => {
                started = true;
                quoted = Some(character);
            }
            None if character.is_whitespace() => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            None => {
                started = true;
                word.push(character);
            }
        }
    }
    if started {
        words.push(word);
    }

    words
}

/// The variables `text` names, a `KEY=value` a line: a line without `=` or
/// with no key names none.
fn variables(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            let key = key.trim();
            (!key.is_empty()).then(|| (key.to_owned(), value.to_owned()))
        })
        .collect()
}

/// What writes the reader's speech down: the provider it reaches, with the
/// base URL and key the request carries.
pub struct Speech {
    /// The provider's base URL, as the settings keep it.
    pub base_url: String,
    /// The key its requests carry.
    pub api_key: String,
}

/// The provider that writes the reader's speech down: DeepSeek, whose
/// speech-to-text the app calls, while one of its providers has a key (tracked
/// where it is read). A proxy that fronts DeepSeek under another host does not
/// count: the request goes to DeepSeek's own speech path.
pub fn speech() -> Option<Speech> {
    settings().with(|settings| {
        settings
            .providers
            .iter()
            .filter(|provider| provider.saved)
            .find(|provider| {
                reaches_deepseek(&provider.base_url) && !provider.api_key.trim().is_empty()
            })
            .map(|provider| Speech {
                base_url: provider.base_url.trim().to_owned(),
                api_key: provider.api_key.trim().to_owned(),
            })
    })
}

/// Whether `base_url` is DeepSeek's own, whatever version path it is kept with.
fn reaches_deepseek(base_url: &str) -> bool {
    let base = base_url.trim().trim_end_matches('/');
    let base = base
        .strip_suffix("/v1")
        .or_else(|| base.strip_suffix("/beta"))
        .unwrap_or(base);

    base == DEEPSEEK_BASE_URL
}

/// A model a reply can ask: its id for the provider, its label for the reader,
/// and the provider that serves it, which the reply asks.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    /// The provider that serves it.
    pub provider: u32,
    /// That provider's name, as the picker's section says it.
    pub provider_name: SharedString,
    /// How that provider's requests are made.
    pub kind: ProviderKind,
    pub id: SharedString,
    pub label: SharedString,
    /// The base URL and key of the provider that serves it.
    pub base_url: String,
    pub api_key: String,
}

/// A local MCP server as the agent starts it: the program, what it is given,
/// and what it runs with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpServer {
    /// What the app calls it, and the prefix of the tools it serves.
    pub name: String,
    /// The program it runs.
    pub command: String,
    /// Its arguments, in order.
    pub args: Vec<String>,
    /// Its environment, on top of the app's.
    pub env: Vec<(String, String)>,
}

/// The settings: its providers, the model the next reply asks, and the keys
/// the next run reads.
#[derive(Clone)]
struct Settings {
    providers: Vec<Provider>,
    /// The id the next provider added takes.
    next_provider_id: u32,
    /// The local MCP servers the reader configured.
    mcp: Vec<Mcp>,
    /// The id the next server added takes.
    next_mcp_id: u32,
    /// The model the next reply asks, by id: the picker's choice, and the one
    /// the chat sends until another is picked.
    selected: SharedString,
    /// The provider that serves it, while the reader picked one: a model id can
    /// be offered by several providers.
    selected_provider: Option<u32>,
    /// How hard that model thinks: the picker's other choice.
    effort: Effort,
    /// How a reply's thinking shows in the conversation.
    thinking: Thinking,
    /// The blank each side of the conversation takes, in percent of its width.
    margin: u32,
    /// The keys kept on disk, by provider id. A provider's key in memory may
    /// come from the environment instead, and is not kept then.
    keys: secrets::Keys,
}

impl Default for Settings {
    /// The settings the app kept between runs, or the ones it starts from.
    fn default() -> Self {
        stored::load()
            .and_then(Settings::restored)
            .unwrap_or_else(Settings::seeded)
    }
}

impl Settings {
    /// The settings `kept` names, while it names a provider at all.
    fn restored(kept: stored::Kept) -> Option<Self> {
        if kept.providers.is_empty() {
            return None;
        }
        let keys = secrets::load();
        let providers = kept
            .providers
            .into_iter()
            .map(|provider| Provider::restored(provider, &keys))
            .collect::<Vec<_>>();
        let next_provider_id = providers
            .iter()
            .map(|provider| provider.id)
            .max()
            .map_or(1, |id| id + 1);
        let mcp = kept.mcp.into_iter().map(Mcp::restored).collect::<Vec<_>>();
        let next_mcp_id = mcp
            .iter()
            .map(|server| server.id)
            .max()
            .map_or(1, |id| id + 1);

        Some(Self {
            providers,
            next_provider_id,
            mcp,
            next_mcp_id,
            selected: kept.selected.into(),
            selected_provider: kept.selected_provider,
            effort: kept.effort.unwrap_or(Effort::High),
            thinking: kept.thinking.unwrap_or_default(),
            margin: kept.margin.unwrap_or(DEFAULT_MARGIN).min(MAX_MARGIN),
            keys,
        })
    }

    /// The settings a new install starts from: DeepSeek, whose key its name's
    /// environment variable may hold. Every other provider is added in the
    /// settings, where its API is chosen.
    fn seeded() -> Self {
        let mut provider = Provider::seeded(
            1,
            "DeepSeek",
            DEEPSEEK_BASE_URL,
            ProviderKind::OpenAiResponses,
        );
        adopt_environment_key(&mut provider);

        Self {
            providers: vec![provider],
            next_provider_id: 2,
            mcp: Vec::new(),
            next_mcp_id: 1,
            selected: DEEPSEEK_MODEL.into(),
            selected_provider: Some(1),
            effort: Effort::High,
            thinking: Thinking::default(),
            margin: DEFAULT_MARGIN,
            keys: secrets::Keys::new(),
        }
    }

    /// What to keep of them: the providers the reader kept, never their keys.
    fn kept(&self) -> stored::Kept {
        stored::Kept {
            providers: self
                .providers
                .iter()
                .filter(|provider| provider.saved)
                .map(|provider| stored::Provider {
                    id: provider.id,
                    name: provider.name.to_string(),
                    kind: provider.kind.kept(),
                    base_url: provider.base_url.clone(),
                    disabled: provider.disabled.iter().map(|id| id.to_string()).collect(),
                })
                .collect(),
            mcp: self
                .mcp
                .iter()
                .filter(|server| server.saved)
                .map(|server| stored::Mcp {
                    id: server.id,
                    name: server.name.to_string(),
                    command: server.command.clone(),
                    args: server.args.clone(),
                    env: server.env.clone(),
                    enabled: server.enabled,
                })
                .collect(),
            selected: self.selected.to_string(),
            selected_provider: self.selected_provider,
            effort: Some(self.effort),
            thinking: Some(self.thinking),
            margin: Some(self.margin),
        }
    }
}

impl Provider {
    /// A provider the app starts from: saved, with no key of its own and no
    /// models yet.
    fn seeded(id: u32, name: &'static str, base_url: &str, kind: ProviderKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            base_url: base_url.to_owned(),
            api_key: String::new(),
            env_key: None,
            disabled: Vec::new(),
            models: Models::Idle,
            ask: 0,
            saved: true,
        }
    }

    /// The provider `kept` names: reachable again with the key it kept, or the
    /// environment's while it keeps none.
    fn restored(kept: stored::Provider, keys: &secrets::Keys) -> Self {
        // A provider the app seeded before its spelling settled keeps that
        // spelling on disk; the name the app started with is the one to show.
        let name = if kept.name == "opencode" {
            "OpenCode".to_owned()
        } else {
            kept.name
        };
        let mut provider = Self {
            id: kept.id,
            name: name.into(),
            kind: ProviderKind::restored(kept.kind, &kept.base_url),
            base_url: kept.base_url,
            api_key: keys.get(&kept.id.to_string()).cloned().unwrap_or_default(),
            env_key: None,
            disabled: kept.disabled.into_iter().map(SharedString::from).collect(),
            models: Models::Idle,
            ask: 0,
            saved: true,
        };
        if provider.api_key.is_empty() {
            adopt_environment_key(&mut provider);
        }

        provider
    }
}

/// Gives `provider` the key its own environment variable holds, and forgets
/// any it held.
fn adopt_environment_key(provider: &mut Provider) {
    provider.api_key.clear();
    provider.env_key = None;
    if let Some((variable, key)) = environment_key(provider.name.as_str()) {
        provider.api_key = key;
        provider.env_key = Some(variable);
    }
}

/// The key the environment variable of `name` holds, with the variable's name:
/// all capitals with everything else one underscore (`DeepSeek` reads
/// `DEEPSEEK_API_KEY`). `None` while it holds none.
fn environment_key(name: &str) -> Option<(SharedString, String)> {
    let mut variable = String::new();
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            variable.push(character.to_ascii_uppercase());
        } else if !variable.is_empty() && !variable.ends_with('_') {
            variable.push('_');
        }
    }
    let variable = variable.trim_end_matches('_');
    if variable.is_empty() {
        return None;
    }
    let variable = format!("{variable}_API_KEY");

    let key = env::var(&variable).ok()?;
    if key.trim().is_empty() {
        return None;
    }

    Some((variable.into(), key))
}

/// The environment variable `id`'s key came from, while it did (tracked where
/// it is read).
pub fn env_key(id: u32) -> Option<SharedString> {
    with_provider(id, |provider| provider.env_key.clone()).flatten()
}

fn settings() -> Signal<Settings> {
    global::<Settings>()
}

/// Keeps the settings as they are now, for the next run.
fn save() {
    settings().with(|settings| stored::save(&settings.kept()));
}

/// Keeps the providers' keys as they are now, for the next run.
fn save_keys() {
    settings().with(|settings| secrets::save(&settings.keys));
}

/// `f` of provider `id`, or none once it is gone (tracked where it is read).
fn with_provider<R>(id: u32, f: impl FnOnce(&Provider) -> R) -> Option<R> {
    settings().with(|settings| settings.providers.iter().find(|p| p.id == id).map(f))
}

/// Provider `id` of `settings`, to change.
fn provider_mut(settings: &mut Settings, id: u32) -> Option<&mut Provider> {
    settings
        .providers
        .iter_mut()
        .find(|provider| provider.id == id)
}

/// Provider `id` of `settings` while it is being added, to change.
fn draft_mut(settings: &mut Settings, id: u32) -> Option<&mut Provider> {
    settings
        .providers
        .iter_mut()
        .find(|provider| provider.id == id && !provider.saved)
}

/// Asks every provider still unasked for its models, so the composer's picker
/// and the settings both have them without a press.
pub fn load() {
    for id in provider_ids() {
        if with_provider(id, |p| matches!(p.models, Models::Idle)).unwrap_or(false) {
            refresh_models(id);
        }
    }
}

/// The models the settings offer: those switched on, in the providers' order
/// (tracked where it is read).
pub fn enabled_models() -> Vec<Choice> {
    settings().with(|settings| {
        let mut choices: Vec<Choice> = Vec::new();
        let offered = settings.providers.iter().filter(|provider| provider.saved);
        for provider in offered {
            for model in provider.models.shown() {
                if model.enabled {
                    choices.push(choice(provider, model));
                }
            }
        }
        choices
    })
}

/// `model` as a reply may ask it, served by `provider`.
fn choice(provider: &Provider, model: &Model) -> Choice {
    Choice {
        provider: provider.id,
        provider_name: provider.name.clone(),
        kind: provider.kind,
        id: model.id.clone(),
        label: model.label.clone(),
        base_url: provider.base_url.clone(),
        api_key: provider.api_key.clone(),
    }
}

/// The model the next reply asks: the one picked, named as its provider does
/// (tracked where it is read). The provider picked serves it; without one, the
/// first provider that lists it does.
pub fn selected() -> Choice {
    let (id, picked) =
        settings().with(|settings| (settings.selected.clone(), settings.selected_provider));
    let found = settings().with(|settings| {
        let serving = |provider: &Provider| {
            provider.saved && provider.models.shown().iter().any(|model| model.id == id)
        };
        let provider = settings
            .providers
            .iter()
            .find(|provider| Some(provider.id) == picked && serving(provider))
            .or_else(|| settings.providers.iter().find(|provider| serving(provider)));
        provider.and_then(|provider| {
            provider
                .models
                .shown()
                .iter()
                .find(|model| model.id == id)
                .map(|model| choice(provider, model))
        })
    });

    found.unwrap_or_else(|| Choice {
        provider: picked.unwrap_or_default(),
        provider_name: SharedString::default(),
        kind: ProviderKind::OpenAiChatCompletions,
        label: id.clone(),
        id,
        base_url: String::new(),
        api_key: String::new(),
    })
}

/// Asks `provider` for `model` from now on.
pub fn select_model(provider: u32, model: &str) {
    let model: SharedString = model.to_owned().into();
    settings().update(|settings| {
        settings.selected = model;
        settings.selected_provider = Some(provider);
    });
    save();
}

/// How hard the next reply's model thinks (tracked where it is read).
pub fn effort() -> Effort {
    settings().with(|settings| settings.effort)
}

/// Asks for `effort` from the replies from now on.
pub fn select_effort(effort: Effort) {
    settings().update(|settings| settings.effort = effort);
    save();
}

/// How a reply's thinking shows (tracked where it is read).
pub fn thinking() -> Thinking {
    settings().with(|settings| settings.thinking)
}

/// Shows a reply's thinking as `thinking` from now on.
pub fn select_thinking(thinking: Thinking) {
    settings().update(|settings| settings.thinking = thinking);
    save();
}

/// The blank each side of the conversation takes, in percent of its width: the
/// column keeps the rest, centered (tracked where it is read).
pub fn margin() -> u32 {
    settings().with(|settings| settings.margin)
}

/// Leaves `percent` of the conversation's width blank on each side.
pub fn select_margin(percent: u32) {
    settings().update(|settings| settings.margin = percent.min(MAX_MARGIN));
    save();
}

/// The providers' ids, in order (tracked where it is read).
pub fn provider_ids() -> Vec<u32> {
    settings().with(|settings| settings.providers.iter().map(|p| p.id).collect())
}

/// The providers the reader kept, in order (tracked where it is read).
pub fn saved_providers() -> Vec<u32> {
    settings().with(|settings| {
        settings
            .providers
            .iter()
            .filter(|p| p.saved)
            .map(|p| p.id)
            .collect()
    })
}

/// The provider being added, while one is (tracked where it is read).
pub fn draft() -> Option<u32> {
    settings().with(|settings| settings.providers.iter().find(|p| !p.saved).map(|p| p.id))
}

/// Starts adding a provider of `kind`, with no name yet. A provider already
/// being added is left as it is.
pub fn add_provider(kind: ProviderKind) {
    settings().update(|settings| {
        if settings.providers.iter().any(|p| !p.saved) {
            return;
        }
        let id = settings.next_provider_id;
        settings.next_provider_id += 1;
        settings.providers.push(Provider {
            id,
            name: SharedString::default(),
            kind,
            base_url: String::new(),
            api_key: String::new(),
            env_key: None,
            disabled: Vec::new(),
            models: Models::Idle,
            ask: 0,
            saved: false,
        });
    });
}

/// Makes `id` speak `kind`. Its name stays what it was: the reader's, or none
/// until they name it.
pub fn select_kind(id: u32, kind: ProviderKind) {
    settings().update(|settings| {
        if let Some(provider) = provider_mut(settings, id) {
            provider.kind = kind;
        }
    });
    save();
}

/// Keeps the provider `id`: it is one of the reader's from now on, and the
/// models it lists are asked for so the composer's picker has them. A key the
/// environment gave stays the environment's: it is not written down.
pub fn save_provider(id: u32) {
    let mut key = None;
    let mut from_environment = false;
    settings().update(|settings| {
        if let Some(provider) = provider_mut(settings, id) {
            provider.saved = true;
            from_environment = provider.env_key.is_some();
            key = Some(provider.api_key.trim().to_owned());
        }
    });
    if let Some(key) = key {
        settings().update(|settings| {
            if from_environment || key.is_empty() {
                settings.keys.remove(&id.to_string());
            } else {
                settings.keys.insert(id.to_string(), key);
            }
        });
        save_keys();
    }
    save();
    load();
}

/// Drops the provider `id` while it is one being added.
pub fn cancel_provider(id: u32) {
    settings().update(|settings| {
        settings.providers.retain(|p| p.id != id || p.saved);
    });
}

/// What `id` is called (tracked where it is read).
pub fn name(id: u32) -> SharedString {
    with_provider(id, |p| p.name.clone()).unwrap_or_default()
}

/// What API `id` speaks, or none once it is gone (tracked where it is read).
pub fn kind(id: u32) -> Option<ProviderKind> {
    with_provider(id, |p| p.kind)
}

/// The URL `id` is reached at (tracked where it is read).
pub fn base_url(id: u32) -> String {
    with_provider(id, |p| p.base_url.clone()).unwrap_or_default()
}

/// The key `id`'s requests carry (tracked where it is read).
pub fn api_key(id: u32) -> String {
    with_provider(id, |p| p.api_key.clone()).unwrap_or_default()
}

/// Names the provider `id`, while it is being added: the key the environment
/// variable of the new name holds takes the field, whatever it held before,
/// and a name whose variable holds none keeps only a key the reader typed.
pub fn set_name(id: u32, name: &str) {
    let name: SharedString = name.to_owned().into();
    settings().update(|settings| {
        let Some(provider) = draft_mut(settings, id) else {
            return;
        };
        provider.name = name;
        match environment_key(provider.name.as_str()) {
            Some((variable, key)) => {
                provider.api_key = key;
                provider.env_key = Some(variable);
            }
            // The key was the environment's for the name it had: the
            // environment holds none for this one.
            None if provider.env_key.is_some() => {
                provider.api_key.clear();
                provider.env_key = None;
            }
            None => {}
        }
    });
}

/// Takes `id`'s key back to the one its name's environment variable holds, or
/// to none when that holds none, whatever the reader typed before.
pub fn reset_api_key(id: u32) {
    let mut forgot = false;
    settings().update(|settings| {
        let Some(provider) = provider_mut(settings, id) else {
            return;
        };
        adopt_environment_key(provider);
        forgot = settings.keys.remove(&id.to_string()).is_some();
    });
    if forgot {
        save_keys();
    }
}

/// Sets the URL `id` is reached at.
pub fn set_base_url(id: u32, url: &str) {
    settings().update(|settings| {
        if let Some(provider) = provider_mut(settings, id) {
            provider.base_url = url.to_owned();
        }
    });
    save();
}

/// Sets the key `id`'s requests carry: the reader's own from now on.
pub fn set_api_key(id: u32, key: &str) {
    let mut saved = false;
    settings().update(|settings| {
        if let Some(provider) = provider_mut(settings, id) {
            provider.api_key = key.to_owned();
            provider.env_key = None;
            saved = provider.saved;
            if saved {
                if key.trim().is_empty() {
                    settings.keys.remove(&id.to_string());
                } else {
                    settings.keys.insert(id.to_string(), key.trim().to_owned());
                }
            }
        }
    });
    if saved {
        save_keys();
    }
}

/// Whether `id` has what an ask needs: a URL and a key.
pub fn can_refresh(id: u32) -> bool {
    with_provider(id, |p| {
        !p.base_url.trim().is_empty() && !p.api_key.trim().is_empty()
    })
    .unwrap_or(false)
}

/// Whether `id` has what it takes to be kept: a name, a URL and a key.
pub fn can_save(id: u32) -> bool {
    with_provider(id, |p| {
        !p.name.trim().is_empty() && !p.base_url.trim().is_empty() && !p.api_key.trim().is_empty()
    })
    .unwrap_or(false)
}

/// Whether an ask for `id` is on its way (tracked where it is read).
pub fn loading(id: u32) -> bool {
    with_provider(id, |p| matches!(p.models, Models::Loading(_))).unwrap_or(false)
}

/// Why the last ask for `id` failed, while it did (tracked where it is read).
pub fn error(id: u32) -> Option<SharedString> {
    with_provider(id, |p| match &p.models {
        Models::Failed(_, why) => Some(why.clone()),
        _ => None,
    })
    .flatten()
}

/// Whether `id` answered with no models at all (tracked where it is read).
pub fn listed_none(id: u32) -> bool {
    with_provider(
        id,
        |p| matches!(&p.models, Models::Ready(models) if models.is_empty()),
    )
    .unwrap_or(false)
}

/// The models `id` shows, in order (tracked where it is read).
pub fn models(id: u32) -> Vec<Model> {
    with_provider(id, |p| p.models.shown().to_vec()).unwrap_or_default()
}

/// Offers the model called `model`, or stops offering it.
pub fn set_model_enabled(id: u32, model: &str, on: bool) {
    let model: SharedString = model.to_owned().into();
    settings().update(|settings| {
        let Some(provider) = provider_mut(settings, id) else {
            return;
        };
        if on {
            provider.disabled.retain(|off| off != &model);
        } else if !provider.disabled.contains(&model) {
            provider.disabled.push(model.clone());
        }
        let (Models::Loading(models) | Models::Ready(models) | Models::Failed(models, _)) =
            &mut provider.models
        else {
            return;
        };
        if let Some(listed) = models.iter_mut().find(|m| m.id == model) {
            listed.enabled = on;
        }
    });
    save();
}

/// Whether every model `id` lists is offered: what its toggle-all shows.
pub fn all_models_enabled(id: u32) -> bool {
    with_provider(id, |provider| {
        let models = provider.models.shown();
        !models.is_empty() && models.iter().all(|model| model.enabled)
    })
    .unwrap_or(false)
}

/// Offers every model `id` lists, or stops offering them all.
pub fn set_all_models_enabled(id: u32, on: bool) {
    settings().update(|settings| {
        let Some(provider) = provider_mut(settings, id) else {
            return;
        };
        let listed: Vec<SharedString> = if on {
            Vec::new()
        } else {
            provider
                .models
                .shown()
                .iter()
                .map(|model| model.id.clone())
                .collect()
        };
        provider.disabled = listed;
        let (Models::Loading(models) | Models::Ready(models) | Models::Failed(models, _)) =
            &mut provider.models
        else {
            return;
        };
        for model in models {
            model.enabled = on;
        }
    });
    save();
}

/// Asks `id` for its models, from the URL and key it holds. Its answer lands
/// in the state; a newer ask makes an older answer drop.
pub fn refresh_models(id: u32) {
    if !can_refresh(id) {
        return;
    }
    let Some((base_url, api_key)) = with_provider(id, |p| {
        (p.base_url.trim().to_owned(), p.api_key.trim().to_owned())
    }) else {
        return;
    };

    let mut asked = 0;
    settings().update(|settings| {
        if let Some(provider) = provider_mut(settings, id) {
            provider.ask += 1;
            let shown = provider.models.shown().to_vec();
            provider.models = Models::Loading(shown);
            asked = provider.ask;
        }
    });
    if asked == 0 {
        return;
    }

    // Under no owner, so closing the dialog does not cut the answer off: it
    // belongs to the provider.
    Owner::detached().run(|| {
        spawn(async move {
            let listed = list_models(&base_url, &api_key).await;
            settings().update(|settings| {
                let Some(provider) = provider_mut(settings, id) else {
                    return;
                };
                if provider.ask != asked {
                    return;
                }
                let shown = provider.models.shown().to_vec();
                let disabled = provider.disabled.clone();
                provider.models = match listed {
                    Ok(listed) => Models::Ready(merged(listed, &disabled)),
                    Err(why) => Models::Failed(shown, why),
                };
            });
        });
    });
}

/// `listed` as the models to show: the reader's switches decide which one is
/// offered, and one they never switched is offered.
fn merged(listed: Vec<ListedModel>, disabled: &[SharedString]) -> Vec<Model> {
    listed
        .into_iter()
        .map(|listed| {
            let id: SharedString = listed.id.into();
            let enabled = !disabled.contains(&id);
            let label = match listed.name {
                Some(name) if !name.trim().is_empty() => name.into(),
                _ => id.clone(),
            };
            Model { id, label, enabled }
        })
        .collect()
}

/// Server `id` of `settings`, to change.
fn mcp_mut(settings: &mut Settings, id: u32) -> Option<&mut Mcp> {
    settings.mcp.iter_mut().find(|server| server.id == id)
}

/// `f` of server `id`, or none once it is gone (tracked where it is read).
fn with_mcp<R>(id: u32, f: impl FnOnce(&Mcp) -> R) -> Option<R> {
    settings().with(|settings| settings.mcp.iter().find(|server| server.id == id).map(f))
}

/// The MCP servers the reader kept, in order (tracked where it is read).
pub fn mcps() -> Vec<u32> {
    settings().with(|settings| {
        settings
            .mcp
            .iter()
            .filter(|server| server.saved)
            .map(|server| server.id)
            .collect()
    })
}

/// The server being added, while one is (tracked where it is read).
pub fn mcp_draft() -> Option<u32> {
    settings().with(|settings| {
        settings
            .mcp
            .iter()
            .find(|server| !server.saved)
            .map(|server| server.id)
    })
}

/// Starts adding a server, with nothing filled in yet. A server already being
/// added is left as it is.
pub fn add_mcp() {
    settings().update(|settings| {
        if settings.mcp.iter().any(|server| !server.saved) {
            return;
        }
        let id = settings.next_mcp_id;
        settings.next_mcp_id += 1;
        settings.mcp.push(Mcp {
            id,
            name: SharedString::default(),
            command: String::new(),
            args: String::new(),
            env: String::new(),
            enabled: true,
            saved: false,
        });
    });
}

/// Keeps the server `id`: it is one of the reader's from now on, and the agent
/// starts it from the next message while it is on.
pub fn save_mcp(id: u32) {
    settings().update(|settings| {
        if let Some(server) = mcp_mut(settings, id) {
            server.saved = true;
        }
    });
    save();
}

/// Drops the server `id` while it is one being added.
pub fn cancel_mcp(id: u32) {
    settings().update(|settings| {
        settings
            .mcp
            .retain(|server| server.id != id || server.saved);
    });
}

/// What `id` is called (tracked where it is read).
pub fn mcp_name(id: u32) -> SharedString {
    with_mcp(id, |server| server.name.clone()).unwrap_or_default()
}

/// The program `id` runs (tracked where it is read).
pub fn mcp_command(id: u32) -> String {
    with_mcp(id, |server| server.command.clone()).unwrap_or_default()
}

/// The arguments `id` was given, as they were typed (tracked where it is read).
pub fn mcp_args(id: u32) -> String {
    with_mcp(id, |server| server.args.clone()).unwrap_or_default()
}

/// The environment `id` was given, as it was typed (tracked where it is read).
pub fn mcp_env(id: u32) -> String {
    with_mcp(id, |server| server.env.clone()).unwrap_or_default()
}

/// Whether the agent starts `id` (tracked where it is read).
pub fn mcp_enabled(id: u32) -> bool {
    with_mcp(id, |server| server.enabled).unwrap_or(false)
}

/// Names the server `id`.
pub fn set_mcp_name(id: u32, name: &str) {
    settings().update(|settings| {
        if let Some(server) = mcp_mut(settings, id) {
            server.name = name.to_owned().into();
        }
    });
    save();
}

/// Sets the program `id` runs.
pub fn set_mcp_command(id: u32, command: &str) {
    settings().update(|settings| {
        if let Some(server) = mcp_mut(settings, id) {
            server.command = command.to_owned();
        }
    });
    save();
}

/// Sets the arguments `id` is given.
pub fn set_mcp_args(id: u32, args: &str) {
    settings().update(|settings| {
        if let Some(server) = mcp_mut(settings, id) {
            server.args = args.to_owned();
        }
    });
    save();
}

/// Sets the environment `id` is given.
pub fn set_mcp_env(id: u32, env: &str) {
    settings().update(|settings| {
        if let Some(server) = mcp_mut(settings, id) {
            server.env = env.to_owned();
        }
    });
    save();
}

/// Starts `id` from the next message on, or stops starting it.
pub fn set_mcp_enabled(id: u32, on: bool) {
    settings().update(|settings| {
        if let Some(server) = mcp_mut(settings, id) {
            server.enabled = on;
        }
    });
    save();
}

/// Whether `id` has what it takes to be kept: a name a server may take, and a
/// command.
pub fn can_save_mcp(id: u32) -> bool {
    with_mcp(id, |server| {
        server_name(server.name.trim()) && !server.command.trim().is_empty()
    })
    .unwrap_or(false)
}

/// Whether `name` is one a server may take: the letters, digits, `_` and `-` a
/// tool name is made of, starting with a letter or `_`, with no `__` and no `_`
/// at its end, as the client that starts it asks. Its tools are named from it.
fn server_name(name: &str) -> bool {
    let starts = name
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_');

    starts
        && !name.contains("__")
        && !name.ends_with('_')
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

/// The servers the agent starts, with their arguments and environment read out
/// of what the reader typed (tracked where it is read).
pub fn enabled_mcps() -> Vec<McpServer> {
    settings().with(|settings| {
        settings
            .mcp
            .iter()
            .filter(|server| server.saved && server.enabled)
            .map(Mcp::choice)
            .filter(|server| !server.command.is_empty() && server_name(&server.name))
            .collect()
    })
}
