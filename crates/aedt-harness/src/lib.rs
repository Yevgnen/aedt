//! The app's agent: rig's harness run as a library, on a thread of its own and
//! talked to over channels. Each session of the app gets an agent of its own,
//! which asks the models the app's providers serve and may call its tools.

mod attach;
mod front;
mod mcp;
mod models;
mod tools;

use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
    thread,
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use rig_harness::prelude::*;

use crate::{front::AgentBridge, models::connector};

pub use crate::models::{Kind, Provider};

/// What the agent reports while a reply happens, for the session whose reply
/// it is.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The chain of thought, as it arrived.
    Thinking(u32, String),
    /// The answer, as it arrived.
    Answer(u32, String),
    /// A tool call started.
    ToolCall {
        session: u32,
        id: String,
        name: String,
        args: String,
    },
    /// A tool call answered.
    ToolResult {
        session: u32,
        id: String,
        name: String,
        ok: bool,
        output: String,
        /// The files the call changed, for a call that edits.
        changes: Vec<ChangedFile>,
    },
    /// The turn failed: this is why.
    Failed(u32, String),
    /// The turn ended: its answer is in.
    Ended(u32),
    /// The session's agent could not run at all, before any of its replies.
    Broken(u32, String),
}

/// What one MCP server of a session is doing, for the composer's status line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpStatus {
    /// The server's name, as the app calls it.
    pub name: String,
    /// Whether its process runs now, and why it does not.
    pub state: McpState,
}

/// A session's MCP server's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum McpState {
    /// Its process is connected and ready for a call.
    Running,
    /// It is configured, and no process runs now.
    Idle,
    /// The last attempt to start it failed.
    Error,
}

/// One file a tool call changed, as the app shows it: what the call says it
/// did, and the change itself.
#[derive(Clone, Debug, PartialEq)]
pub struct ChangedFile {
    /// The file, as the call named it.
    pub path: String,
    /// Lines it added.
    pub added: usize,
    /// Lines it removed.
    pub removed: usize,
    /// The change as a unified diff, its lines each prefixed by the sign that
    /// says what it does; empty for a change the tool did not diff.
    pub diff: String,
}

/// What the app tells the agent.
#[derive(Clone, Debug)]
pub(crate) enum Command {
    /// Say `text` in `session`, with the files the reader attached, asking
    /// `model` (`vendor/model`); `before` is the conversation the session
    /// already had, which an agent made now must know, and `root` the folder it
    /// works in, while its workspace gives one.
    Say {
        session: u32,
        model: String,
        text: String,
        files: Vec<PathBuf>,
        before: Vec<Turn>,
        root: Option<std::path::PathBuf>,
    },
    /// Ask `model` in `session` from now on.
    SetModel { session: u32, model: String },
}

/// One turn of a conversation, before the message being sent.
#[derive(Clone, Debug)]
pub struct Turn {
    /// Whether the reader wrote it; an assistant's turn is not.
    pub user: bool,
    pub text: String,
}

/// What the agent needs to run.
#[derive(Clone, Debug)]
pub struct Config {
    /// The session this harness answers for: its agent, its tools and its MCP
    /// servers are its own.
    pub session: u32,
    /// The providers the app offers, each with the models the reader left on.
    pub providers: Vec<Provider>,
    /// The local MCP servers the app switched on, their tools listed when the
    /// harness asks for them.
    pub mcp: Vec<McpServer>,
    /// Where each server's catalog is kept: the tools it listed last time, one
    /// file a server.
    pub catalog: PathBuf,
    /// The directory the tools work in.
    pub root: PathBuf,
    /// The harness's own home: its sessions, logs and settings.
    pub home: PathBuf,
}

/// A local MCP server as the app configures it: a program to run, what it is
/// given, and what it runs with.
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

/// The app's handle on the agent: what it says, and what comes back.
pub struct Agent {
    commands: Sender<Command>,
    events: Receiver<Event>,
    /// The servers this agent's harness was made from, once it has prepared
    /// them, for the composer's status line.
    mcp: Arc<OnceLock<Arc<Vec<mcp::Prepared>>>>,
}

impl Agent {
    /// Starts the harness on a thread of its own.
    ///
    /// # Errors
    ///
    /// Why the thread could not be started.
    pub fn start(config: Config) -> Result<Agent, String> {
        let (commands, inbox) = unbounded();
        let (outbox, events) = unbounded();
        let mcp = Arc::new(OnceLock::new());
        let slot = Arc::clone(&mcp);
        let name = format!("aedt-harness-{}", config.session);
        thread::Builder::new()
            .name(name)
            .spawn(move || run(config, inbox, outbox, slot))
            .map_err(|error| format!("failed to start the agent thread: {error}"))?;
        Ok(Agent {
            commands,
            events,
            mcp,
        })
    }

    /// Says `text` in `session`, with the files the reader attached, asking
    /// `model` (`vendor/model`), with the turns the session already had and the
    /// folder it works in.
    pub fn say(
        &self,
        session: u32,
        model: &str,
        text: &str,
        files: Vec<PathBuf>,
        before: Vec<Turn>,
        root: Option<std::path::PathBuf>,
    ) {
        let _ = self.commands.send(Command::Say {
            session,
            model: model.to_owned(),
            text: text.to_owned(),
            files,
            before,
            root,
        });
    }

    /// Asks `model` in `session` from now on.
    pub fn set_model(&self, session: u32, model: &str) {
        let _ = self.commands.send(Command::SetModel {
            session,
            model: model.to_owned(),
        });
    }

    /// The MCP servers this agent was made from, and what each is doing now.
    pub fn mcp_status(&self) -> Vec<McpStatus> {
        self.mcp
            .get()
            .map(|prepared| prepared.iter().map(mcp::Prepared::status).collect())
            .unwrap_or_default()
    }

    /// What the agent reported since the last call, if anything.
    pub fn poll(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }
}

/// Runs the harness until the process ends.
fn run(
    config: Config,
    commands: Receiver<Command>,
    events: Sender<Event>,
    mcp_slot: Arc<OnceLock<Arc<Vec<mcp::Prepared>>>>,
) {
    let models = match connector(&config.providers) {
        Ok(models) => models,
        Err(why) => {
            let _ = events.send(Event::Broken(config.session, why));
            return;
        }
    };
    // The MCP servers' tools, listed when this harness asks for them and
    // shared with the harnesses after it: a process starts for a list or a call
    // and is ended once idle.
    let mcp = Arc::new(mcp::prepare(&config.mcp, &config.catalog));
    let _ = mcp_slot.set(Arc::clone(&mcp));

    let mut app = App::new();
    // A failing system, observer or command from a plugin is logged.
    app.set_error_handler(rig_harness::error::warn);
    app.insert_resource(Models(models))
        .insert_resource(ProjectRoot(config.root))
        .insert_resource(RigHome(Home::at(config.home)))
        .add_plugins((
            HeadlessPlugins,
            RigHarnessPlugins,
            rig_basics::prompt::DefaultPromptPlugin,
            tools::CodingTools,
            mcp::McpTools::new(mcp),
            AgentBridge::new(commands, events),
        ));
    rig_harness::run(app);
}
