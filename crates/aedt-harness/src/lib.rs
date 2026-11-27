//! The app's agent: rig's harness run as a library, on a thread of its own and
//! talked to over channels. Each session of the app gets an agent of its own,
//! which asks the models the app's providers serve and may call its tools.

mod front;
mod mcp;
mod models;
mod tools;

use std::{path::PathBuf, thread};

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
    /// Say `text` in `session`, asking `model` (`vendor/model`); `before` is
    /// the conversation the session already had, which an agent made now must
    /// know, and `root` the folder it works in, while its workspace gives
    /// one.
    Say {
        session: u32,
        model: String,
        text: String,
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
    /// The local MCP servers the app switched on, their tools offered before
    /// any of them runs.
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
        let name = format!("aedt-harness-{}", config.session);
        thread::Builder::new()
            .name(name)
            .spawn(move || run(config, inbox, outbox))
            .map_err(|error| format!("failed to start the agent thread: {error}"))?;
        Ok(Agent { commands, events })
    }

    /// Says `text` in `session`, asking `model` (`vendor/model`), with the
    /// turns the session already had and the folder it works in.
    pub fn say(
        &self,
        session: u32,
        model: &str,
        text: &str,
        before: Vec<Turn>,
        root: Option<std::path::PathBuf>,
    ) {
        let _ = self.commands.send(Command::Say {
            session,
            model: model.to_owned(),
            text: text.to_owned(),
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

    /// What the agent reported since the last call, if anything.
    pub fn poll(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }
}

/// Runs the harness until the process ends.
fn run(config: Config, commands: Receiver<Command>, events: Sender<Event>) {
    let models = match connector(&config.providers) {
        Ok(models) => models,
        Err(why) => {
            let _ = events.send(Event::Broken(config.session, why));
            return;
        }
    };
    // The MCP servers' tools, offered from what they listed last time: nothing
    // of theirs runs until a call needs it.
    let mcp = mcp::prepare(&config.mcp, &config.catalog);

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
