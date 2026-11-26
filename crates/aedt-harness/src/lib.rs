//! The app's agent: rig's harness run as a library, on a thread of its own and
//! talked to over channels. Each session of the app gets an agent of its own,
//! which asks the models the app's providers serve and may call its tools.

mod front;
mod models;

use std::{path::PathBuf, thread};

use crossbeam_channel::{Receiver, Sender, unbounded};
use rig_harness::prelude::*;

use crate::{front::Front, models::connector};

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
    },
    /// The turn failed: this is why.
    Failed(u32, String),
    /// The turn ended: its answer is in.
    Ended(u32),
    /// The agent could not run at all, before any session asked.
    Broken(String),
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
    /// The providers the app offers, each with the models the reader left on.
    pub providers: Vec<Provider>,
    /// The directory the tools work in.
    pub root: PathBuf,
    /// The harness's own home: its sessions, logs and settings.
    pub home: PathBuf,
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
        thread::Builder::new()
            .name("aedt-harness".to_owned())
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
            let _ = events.send(Event::Broken(why));
            return;
        }
    };

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
            // The tools a reply may call: the ones that read what the agent
            // works on, and one that runs commands there.
            rig_basics::coding_tools::ReadTool,
            rig_basics::coding_tools::ShellTool,
            Front::new(commands, events),
        ));
    rig_harness::run(app);
}
