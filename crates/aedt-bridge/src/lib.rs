//! The chat: a reply is asked of the agent the harness runs, and what it
//! streams — its thinking, its tool calls and their results, its answer — is
//! shown as it arrives.

use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    sync::OnceLock,
    time::{Duration, Instant},
};

use aedt_harness::{Agent, Config, Event, Kind, McpServer, Provider, Turn};
use aedt_settings as settings;
use aedt_store::{self as store, Author};
use nsh_core::prelude::*;

pub mod voice;

/// Whether the task that reads what the agents report was started.
static DISPATCH: OnceLock<()> = OnceLock::new();

/// The agent of one session, with the providers and the MCP servers it was
/// started from.
type Running = (Vec<Provider>, Vec<McpServer>, Agent);

thread_local! {
    /// One agent per session, each with its own harness; the MCP servers behind
    /// them are shared, one process for every session. The settings that build
    /// one change while the app runs, and a session's agent is started again
    /// from them on its next message.
    static AGENTS: RefCell<HashMap<u32, Running>> = RefCell::new(HashMap::new());

    /// When each session's reply was asked, and the model it asked, for its
    /// duration and its meta line.
    static STARTED: RefCell<HashMap<u32, (Instant, SharedString)>> = RefCell::new(HashMap::new());
}

/// Sends `text` to the conversation, with the files the reader attached, and
/// lets the agent answer it: the store keeps the conversation, the settings name
/// the model and its provider.
pub fn ask(session_id: u32, text: &str, files: &[PathBuf]) {
    let text = text.trim();
    if text.is_empty() && files.is_empty() {
        return;
    }

    // The agent answers on the providers the settings hold now, started again
    // before this message is in the store: only a reply the agent being
    // replaced was still writing is what fails then.
    let ready = ensure_agent(session_id);

    let Some(sent) = store::send(session_id, &with_paths(text, files)) else {
        return;
    };
    let model = settings::selected();
    if model.base_url.trim().is_empty() || model.api_key.trim().is_empty() {
        store::fail_stream(session_id, "API URL or API key is missing");
        return;
    }
    if let Err(why) = ready {
        store::fail_stream(session_id, &format!("the agent could not start: {why}"));
        return;
    }

    // What came before the message just sent: an agent made for this session
    // reads it before it answers.
    let before = sent
        .history
        .iter()
        .take(sent.history.len().saturating_sub(1))
        .filter(|message| !message.body().is_empty())
        .map(|message| Turn {
            user: message.author == Author::User,
            text: message.body(),
        })
        .collect();

    let reference = format!("{}/{}", model.provider_name, model.id);
    let root = store::session_path(session_id).map(|path| std::path::PathBuf::from(path.as_str()));
    STARTED.with(|started| {
        started
            .borrow_mut()
            .insert(session_id, (Instant::now(), model.label.clone()));
    });
    say(session_id, &reference, text, files, before, root);
}

/// `text` with each attached file's path on a line of its own, under a blank
/// line: what the transcript keeps, so the message reads where each file came
/// from even when the model does not read it.
fn with_paths(text: &str, files: &[PathBuf]) -> String {
    let paths = files
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    match (text.is_empty(), paths.is_empty()) {
        (_, true) => text.to_owned(),
        (true, false) => paths.join("\n"),
        (false, false) => format!("{text}\n\n{}", paths.join("\n")),
    }
}

/// Makes `session`'s agent the one the settings hold: started while it has
/// none, and started again while the providers or the MCP servers it was made
/// from changed. `Err` says why none could start.
fn ensure_agent(session: u32) -> Result<(), String> {
    let providers = providers()?;
    let mcp = mcps();
    let changed = AGENTS.with(|agents| {
        let agents = agents.borrow();

        !agents
            .get(&session)
            .is_some_and(|(made, made_mcp, _)| *made == providers && *made_mcp == mcp)
    });
    if !changed {
        return Ok(());
    }

    // A reply the agent being replaced is still writing cannot finish. The
    // harness goes with it, and the servers it started.
    if AGENTS
        .with(|agents| agents.borrow_mut().remove(&session))
        .is_some()
    {
        store::fail_stream(session, "the settings the agent runs from changed");
        STARTED.with(|started| started.borrow_mut().remove(&session));
    }

    let started = start(session, &providers, &mcp)?;
    AGENTS.with(|agents| {
        agents
            .borrow_mut()
            .insert(session, (providers, mcp, started))
    });
    Ok(())
}

/// Sends the prompt to the session's agent as it is.
fn say(
    session_id: u32,
    model: &str,
    text: &str,
    files: &[PathBuf],
    before: Vec<Turn>,
    root: Option<PathBuf>,
) {
    AGENTS.with(|agents| {
        if let Some((_, _, agent)) = agents.borrow().get(&session_id) {
            agent.say(session_id, model, text, files.to_vec(), before, root);
        }
    });
}

/// The providers the settings offer, as the harness takes them: one per
/// provider with a model left on, reached the way its settings say.
fn providers() -> Result<Vec<Provider>, String> {
    let mut providers: Vec<Provider> = Vec::new();
    for choice in settings::enabled_models() {
        let kind = match choice.kind {
            settings::ProviderKind::OpenAiResponses => Kind::Responses,
            settings::ProviderKind::OpenAiChatCompletions => Kind::ChatCompletions,
        };
        let name = choice.provider_name.to_string();
        match providers.iter_mut().find(|provider| provider.name == name) {
            Some(provider) => provider.models.push(choice.id.to_string()),
            None => providers.push(Provider {
                name,
                base_url: choice.base_url.clone(),
                api_key: choice.api_key.clone(),
                kind,
                models: vec![choice.id.to_string()],
            }),
        }
    }
    if providers.is_empty() {
        return Err("no provider offers a model, so add one in the settings".to_owned());
    }

    Ok(providers)
}

/// The MCP servers the settings switched on, as the harness starts them.
fn mcps() -> Vec<McpServer> {
    settings::enabled_mcps()
        .into_iter()
        .map(|server| McpServer {
            name: server.name,
            command: server.command,
            args: server.args,
            env: server.env,
        })
        .collect()
}

/// Starts `session`'s harness with `providers` and the MCP servers `mcp`, and
/// once the task that reports what it does.
fn start(session: u32, providers: &[Provider], mcp: &[McpServer]) -> Result<Agent, String> {
    let root = std::env::current_dir()
        .map_err(|error| format!("failed to read the working directory: {error}"))?;
    // The harness's own home, one per session: no two harnesses write one
    // session log.
    let rig = nsh_core::element::data_dir()
        .ok_or_else(|| "the app's data directory is unknown".to_owned())?
        .join("rig");
    let home = rig.join(format!("session-{session}"));
    let agent = Agent::start(Config {
        session,
        providers: providers.to_vec(),
        mcp: mcp.to_vec(),
        // The catalogs are every session's: a server is listed once for all.
        catalog: rig.join("mcp"),
        root,
        home,
    })?;
    DISPATCH.get_or_init(|| {
        Owner::detached().run(|| spawn(dispatch()));
    });

    Ok(agent)
}

/// Reports what the agents do, for as long as the app runs: it runs under no
/// owner, so no pane going away stops it.
async fn dispatch() {
    loop {
        let events: Vec<Event> = AGENTS.with(|agents| {
            let agents = agents.borrow();

            agents
                .values()
                .flat_map(|(_, _, agent)| std::iter::from_fn(|| agent.poll()))
                .collect()
        });
        for event in events {
            apply(event);
        }
        timer(Duration::from_millis(16)).await;
    }
}

/// Shows one thing the agent reported.
fn apply(event: Event) {
    match event {
        Event::Thinking(session, text) => store::append_stream(session, &text, ""),
        Event::Answer(session, text) => store::append_stream(session, "", &text),
        Event::ToolCall {
            session,
            id,
            name,
            args,
        } => store::tool_started(session, &id, &name, &args),
        Event::ToolResult {
            session,
            id,
            output,
            ok,
            changes,
            ..
        } => store::tool_finished(
            session,
            &id,
            &output,
            !ok,
            changes
                .into_iter()
                .map(|change| store::ToolChange {
                    path: change.path.into(),
                    added: change.added,
                    removed: change.removed,
                    diff: change.diff.into(),
                })
                .collect(),
        ),
        Event::Ended(session) => {
            let started = STARTED.with(|started| started.borrow_mut().remove(&session));
            let (seconds, model) = started.map_or((0.0, SharedString::default()), |(at, model)| {
                (at.elapsed().as_secs_f32(), model)
            });
            store::finish_stream(session, model.as_str(), seconds);
        }
        Event::Failed(session, why) => {
            STARTED.with(|started| started.borrow_mut().remove(&session));
            store::fail_stream(session, &why);
        }
        // This session's agent cannot run at all: no answer is coming for it.
        Event::Broken(session, why) => {
            eprintln!("agent: {why}");
            STARTED.with(|started| started.borrow_mut().remove(&session));
            store::fail_stream(session, &why);
        }
    }
}
