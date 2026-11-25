//! The app's settings: its model providers, and the models each one serves.
//! The providers are the app's state; asking one for its models fills them
//! from its own `/models`, so nothing here is typed by hand.

use std::env;

use nsh_core::{platform::HttpRequest, prelude::*};
use serde::Deserialize;

/// A provider's API: the shape of its requests and of its model list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProviderKind {
    /// OpenAI's API, as other providers serve it: `/models` lists the models.
    OpenAiCompatible,
}

/// One model a provider serves.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Model {
    /// Its id, as the provider names it: what a chat asks for.
    pub(crate) id: SharedString,
    /// What the app calls it: the provider's own name for it, or its id when
    /// the provider gives none.
    pub(crate) label: SharedString,
    /// Whether the app offers it; everything a provider lists starts offered.
    pub(crate) enabled: bool,
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
    models: Models,
    /// Which ask the last answer belongs to: an older answer does not land.
    ask: u64,
}

/// A model a reply can ask: its id for the provider, its label for the reader.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Choice {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
}

/// How hard a reply's model thinks before it answers (`reasoning.effort`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effort {
    /// The provider's own default: the request asks for no effort.
    Default,
    /// A little.
    Low,
    /// Some: more than the low effort, less than the high one.
    Medium,
    /// As much as the provider offers.
    High,
}

/// A category of the settings: a row of its dialog's left column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Section {
    /// The model providers and their models.
    Models,
}

/// The settings: whether they show, which category does, its providers and the
/// model the next reply asks.
#[derive(Clone)]
struct Settings {
    open: bool,
    section: Section,
    providers: Vec<Provider>,
    /// The model the next reply asks, by id: the picker's choice, and the one
    /// the chat sends until another is picked.
    selected: SharedString,
    /// How hard that model thinks: the picker's other choice.
    effort: Effort,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            open: false,
            section: Section::Models,
            // DeepSeek, whose key the app's chat already reads: its models
            // arrive as soon as the settings open with that key set.
            providers: vec![Provider {
                id: 1,
                name: "DeepSeek".into(),
                kind: ProviderKind::OpenAiCompatible,
                base_url: crate::llm::BASE_URL.to_owned(),
                api_key: env::var(crate::llm::API_KEY).unwrap_or_default(),
                models: Models::Idle,
                ask: 0,
            }],
            selected: crate::llm::MODEL.into(),
            effort: Effort::High,
        }
    }
}

fn settings() -> Signal<Settings> {
    global::<Settings>()
}

/// `f` of provider `id`, or none once it is gone (tracked where it is read).
fn with_provider<R>(id: u32, f: impl FnOnce(&Provider) -> R) -> Option<R> {
    settings().with(|settings| settings.providers.iter().find(|p| p.id == id).map(f))
}

/// Whether the settings dialog shows (tracked where it is read).
pub(crate) fn is_open() -> bool {
    settings().with(|settings| settings.open)
}

/// Shows the settings, and asks each provider that has not listed its models
/// yet: what the app can know of them fills in without a press.
pub(crate) fn open() {
    settings().update(|settings| settings.open = true);
    load();
}

/// Asks every provider still unasked for its models, so the composer's picker
/// and the settings both have them without a press.
pub(crate) fn load() {
    for id in provider_ids() {
        if with_provider(id, |p| matches!(p.models, Models::Idle)).unwrap_or(false) {
            refresh_models(id);
        }
    }
}

/// The models the settings offer: those switched on, in the providers' order
/// (tracked where it is read).
pub(crate) fn enabled_models() -> Vec<Choice> {
    settings().with(|settings| {
        let mut choices: Vec<Choice> = Vec::new();
        for model in settings.providers.iter().flat_map(|p| p.models.shown()) {
            if !model.enabled || choices.iter().any(|choice| choice.id == model.id) {
                continue;
            }
            choices.push(Choice {
                id: model.id.clone(),
                label: model.label.clone(),
            });
        }
        choices
    })
}

/// The model the next reply asks: the one picked, named as its provider does
/// (tracked where it is read).
pub(crate) fn selected() -> Choice {
    let id = settings().with(|settings| settings.selected.clone());
    let known = settings().with(|settings| {
        settings
            .providers
            .iter()
            .flat_map(|p| p.models.shown())
            .find(|model| model.id == id)
            .map(|model| Choice {
                id: model.id.clone(),
                label: model.label.clone(),
            })
    });
    known.unwrap_or_else(|| Choice {
        label: id.clone(),
        id,
    })
}

/// Asks `model` for the replies from now on.
pub(crate) fn select_model(model: &str) {
    let model: SharedString = model.to_owned().into();
    settings().update(|settings| settings.selected = model.clone());
}

/// How hard the next reply's model thinks (tracked where it is read).
pub(crate) fn effort() -> Effort {
    settings().with(|settings| settings.effort)
}

/// Asks for `effort` from the replies from now on.
pub(crate) fn select_effort(effort: Effort) {
    settings().update(|settings| settings.effort = effort);
}

/// Hides the settings.
pub(crate) fn close() {
    settings().update(|settings| settings.open = false);
}

/// The category the dialog shows (tracked where it is read).
pub(crate) fn section() -> Section {
    settings().with(|settings| settings.section)
}

/// Shows `section` in the dialog.
pub(crate) fn select_section(next: Section) {
    settings().update(|settings| settings.section = next);
}

/// The providers' ids, in order (tracked where it is read).
pub(crate) fn provider_ids() -> Vec<u32> {
    settings().with(|settings| settings.providers.iter().map(|p| p.id).collect())
}

/// What `id` is called (tracked where it is read).
pub(crate) fn name(id: u32) -> SharedString {
    with_provider(id, |p| p.name.clone()).unwrap_or_default()
}

/// What API `id` speaks, or none once it is gone (tracked where it is read).
pub(crate) fn kind(id: u32) -> Option<ProviderKind> {
    with_provider(id, |p| p.kind)
}

/// The URL `id` is reached at (tracked where it is read).
pub(crate) fn base_url(id: u32) -> String {
    with_provider(id, |p| p.base_url.clone()).unwrap_or_default()
}

/// The key `id`'s requests carry (tracked where it is read).
pub(crate) fn api_key(id: u32) -> String {
    with_provider(id, |p| p.api_key.clone()).unwrap_or_default()
}

/// Sets the URL `id` is reached at.
pub(crate) fn set_base_url(id: u32, url: &str) {
    settings().update(|settings| {
        if let Some(provider) = settings.providers.iter_mut().find(|p| p.id == id) {
            provider.base_url = url.to_owned();
        }
    });
}

/// Sets the key `id`'s requests carry.
pub(crate) fn set_api_key(id: u32, key: &str) {
    settings().update(|settings| {
        if let Some(provider) = settings.providers.iter_mut().find(|p| p.id == id) {
            provider.api_key = key.to_owned();
        }
    });
}

/// Whether `id` has what an ask needs: a URL and a key.
pub(crate) fn can_refresh(id: u32) -> bool {
    with_provider(id, |p| {
        !p.base_url.trim().is_empty() && !p.api_key.trim().is_empty()
    })
    .unwrap_or(false)
}

/// Whether an ask for `id` is on its way (tracked where it is read).
pub(crate) fn loading(id: u32) -> bool {
    with_provider(id, |p| matches!(p.models, Models::Loading(_))).unwrap_or(false)
}

/// Why the last ask for `id` failed, while it did (tracked where it is read).
pub(crate) fn error(id: u32) -> Option<SharedString> {
    with_provider(id, |p| match &p.models {
        Models::Failed(_, why) => Some(why.clone()),
        _ => None,
    })
    .flatten()
}

/// Whether `id` answered with no models at all (tracked where it is read).
pub(crate) fn listed_none(id: u32) -> bool {
    with_provider(
        id,
        |p| matches!(&p.models, Models::Ready(models) if models.is_empty()),
    )
    .unwrap_or(false)
}

/// The models `id` shows, in order (tracked where it is read).
pub(crate) fn models(id: u32) -> Vec<Model> {
    with_provider(id, |p| p.models.shown().to_vec()).unwrap_or_default()
}

/// Offers the model called `model`, or stops offering it.
pub(crate) fn set_model_enabled(id: u32, model: &str, on: bool) {
    settings().update(|settings| {
        let Some(provider) = settings.providers.iter_mut().find(|p| p.id == id) else {
            return;
        };
        let (Models::Loading(models) | Models::Ready(models) | Models::Failed(models, _)) =
            &mut provider.models
        else {
            return;
        };
        if let Some(model) = models.iter_mut().find(|m| m.id.as_str() == model) {
            model.enabled = on;
        }
    });
}

/// Asks `id` for its models, from the URL and key it holds. Its answer lands
/// in the state; a newer ask makes an older answer drop.
pub(crate) fn refresh_models(id: u32) {
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
        if let Some(provider) = settings.providers.iter_mut().find(|p| p.id == id) {
            provider.ask += 1;
            let shown = provider.models.shown().to_vec();
            provider.models = Models::Loading(shown);
            asked = provider.ask;
        }
    });
    if asked == 0 {
        return;
    }

    // Under no owner, so switching sections or closing the settings does not
    // cut the answer off: it belongs to the provider.
    Owner::detached().run(|| {
        spawn(async move {
            let listed = list_models(&base_url, &api_key).await;
            settings().update(|settings| {
                let Some(provider) = settings.providers.iter_mut().find(|p| p.id == id) else {
                    return;
                };
                if provider.ask != asked {
                    return;
                }
                let shown = provider.models.shown().to_vec();
                provider.models = match listed {
                    Ok(ids) => Models::Ready(merged(&shown, ids)),
                    Err(why) => Models::Failed(shown, why),
                };
            });
        });
    });
}

/// `listed` as the models to show, keeping the choice made of the models
/// shown before: one it did not have joins offered.
fn merged(shown: &[Model], listed: Vec<ListedModel>) -> Vec<Model> {
    listed
        .into_iter()
        .map(|listed| {
            let enabled = shown
                .iter()
                .find(|m| m.id.as_str() == listed.id.as_str())
                .is_none_or(|m| m.enabled);
            let id: SharedString = listed.id.into();
            let label = match listed.name {
                Some(name) if !name.trim().is_empty() => name.into(),
                _ => id.clone(),
            };
            Model { id, label, enabled }
        })
        .collect()
}

/// The models `base_url` lists, by id, in order. It reads the answer's `data`
/// and each model's `id` only, so whatever else a provider adds or leaves out
/// of a model does not fail the whole list.
async fn list_models(base_url: &str, api_key: &str) -> Result<Vec<ListedModel>, SharedString> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let request = HttpRequest::get(url).header("Authorization", format!("Bearer {api_key}"));
    let response = fetch(request).await.map_err(SharedString::from)?;
    let body = response.text();
    if !response.ok() {
        return Err(failure(response.status(), &body).into());
    }

    let listed: Listed = serde_json::from_str(&body)
        .map_err(|error| SharedString::from(format!("Couldn't read the model list: {error}")))?;
    let mut models = listed.data;
    models.sort_by(|a, b| a.id.cmp(&b.id));
    models.dedup_by(|a, b| a.id == b.id);
    Ok(models)
}

/// What a refused answer says: its status, and the API's own message when the
/// body carries one.
fn failure(status: u16, body: &str) -> String {
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
    data: Vec<ListedModel>,
}

/// One model of that answer, of what the app reads.
#[derive(Deserialize)]
struct ListedModel {
    id: String,
    /// The provider's own name for it (DeepSeek gives one, `OpenAI` does not).
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
