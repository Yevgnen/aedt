//! The app's providers as the harness's model catalog: each provider is a
//! vendor rig has no preset for, reached through an OpenAI client of its own,
//! on the API the settings chose for it.

use rig_harness::rig_core::catalog::{Catalog, Connector, ModelSpec, Transport};
use rig_harness::rig_core::driver::DynModel;
use rig_harness::rig_core::operation::Completion;
use rig_harness::rig_core::providers::openai::OpenAIConfig;
use rig_harness::rig_core::providers::registry::ProviderId;
use rig_harness::rig_core::serve::ErasedHandler;
use rig_harness::rig_core::serve::adapters::ModelAdapter;

/// How a provider's requests are made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// OpenAI's Responses API: `POST /responses`.
    Responses,
    /// OpenAI's chat completions: `POST /chat/completions`.
    ChatCompletions,
}

/// One provider the app offers, with the models a reply may ask for.
#[derive(Clone, Debug, PartialEq)]
pub struct Provider {
    /// Its name, which is the catalog's vendor: no spaces or slashes.
    pub name: String,
    /// Where its requests go.
    pub base_url: String,
    /// The key they carry.
    pub api_key: String,
    /// How they are shaped.
    pub kind: Kind,
    /// The models it offers, by its own ids.
    pub models: Vec<String>,
}

/// The catalog the harness asks: `provider/model` references, each reached
/// through its provider's own address and key.
pub(crate) fn connector(providers: &[Provider]) -> Result<Connector, String> {
    let mut connector = Connector::default();
    for provider in providers {
        let vendor = ProviderId::declare(&provider.name)
            .map_err(|error| format!("invalid provider name {:?}: {error}", provider.name))?;

        let base_url = provider.base_url.trim_end_matches('/').to_owned();
        let api_key = provider.api_key.clone();
        let kind = provider.kind;
        connector.provide(
            vendor,
            move |spec: &ModelSpec, _catalog: &Catalog, _transport: &Transport| {
                let config = OpenAIConfig::new(api_key.clone()).with_base_url(base_url.clone());
                let client = config.client();
                let model: DynModel<Completion> = match kind {
                    Kind::Responses => client.responses(spec.id.clone()).into(),
                    Kind::ChatCompletions => client.chat(spec.id.clone()).into(),
                };
                Ok(ErasedHandler::new(ModelAdapter::<Completion>::new(
                    spec.reference(),
                    model,
                )))
            },
        );

        for model in &provider.models {
            connector
                .catalog_mut()
                .insert(ModelSpec::new(vendor, model.clone()).with_tools(true));
        }
    }
    Ok(connector)
}
