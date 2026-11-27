//! The app's front of the harness: it makes an agent per session, hands it
//! what the reader writes, and reports what it does over the channels. The
//! harness's own views mean nothing here, so no terminal or print front runs.

use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::Mutex,
};

use bevy_ecs::message::MessageCursor;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use rig_harness::prelude::Incoming as RigIncoming;
use rig_harness::prelude::*;

use crate::{Command, Event, Turn, attach};

mod events;

use events::{flush, model_refused, tool_settled, tool_started};

/// The front, made once: the channels it reads and writes.
pub(crate) struct AgentBridge {
    commands: Mutex<Option<Receiver<Command>>>,
    events: Sender<Event>,
}

impl AgentBridge {
    pub(crate) fn new(commands: Receiver<Command>, events: Sender<Event>) -> Self {
        Self {
            commands: Mutex::new(Some(commands)),
            events,
        }
    }
}

impl Plugin for AgentBridge {
    fn build(&self, app: &mut App) {
        let commands = self
            .commands
            .lock()
            .ok()
            .and_then(|mut commands| commands.take());
        if let Some(commands) = commands {
            app.insert_resource(Incoming(commands));
        }
        app.insert_resource(Events(self.events.clone()))
            .init_resource::<Sessions>()
            .init_resource::<Waiting>()
            .init_resource::<Calls>()
            .init_resource::<Read>()
            // Before the loop, so a prompt the reader wrote goes in this frame.
            .add_systems(Update, (receive, deliver).chain().before(AgentLoop))
            // After it, so the text a call streamed this frame is out before
            // the next frame's tool events.
            .add_systems(PostUpdate, flush)
            .add_observer(tool_started)
            .add_observer(tool_settled)
            .add_observer(model_refused);
    }
}

/// What the app says, read by the frame.
#[derive(Resource)]
pub(super) struct Incoming(pub(super) Receiver<Command>);

#[derive(Resource)]
pub(super) struct Events(pub(super) Sender<Event>);

#[derive(Resource, Default)]
pub(super) struct Sessions {
    agents: HashMap<u32, Entity>,
    sessions: HashMap<Entity, u32>,
}

impl Sessions {
    /// The session `agent` answers for, while it is one of ours.
    pub(super) fn session(&self, agent: Entity) -> Option<u32> {
        self.sessions.get(&agent).copied()
    }

    /// The agent `session` talks to, once it has one.
    fn agent(&self, session: u32) -> Option<Entity> {
        self.agents.get(&session).copied()
    }

    /// Makes `agent` the one `session` talks to.
    fn bind(&mut self, session: u32, agent: Entity) {
        self.agents.insert(session, agent);
        self.sessions.insert(agent, session);
    }
}

/// The prompts waiting for their agent to connect, each with the conversation
/// its agent must know first, and the files it carries, while it is new.
#[derive(Resource, Default)]
pub(super) struct Waiting(pub(super) VecDeque<Pending>);

/// A prompt the reader sent before its session's agent was connected.
pub(super) struct Pending {
    pub(super) session: u32,
    /// What a new agent is told of the conversation before this message.
    pub(super) note: Option<String>,
    pub(super) text: String,
    /// The files the message carries, as the reader picked them.
    pub(super) files: Vec<PathBuf>,
}

impl Pending {
    /// The message its agent takes: the reader's words, then each file's path
    /// and, for an image, the image itself.
    pub(super) fn message(&self) -> RigIncoming {
        let mut message = RigIncoming::new(self.text.trim(), DeliveryMode::Steer);
        for file in &self.files {
            // The path goes in every time, so the model reads where the file
            // is even when its content does not follow.
            let label = label(file);
            message.parts.push(MessagePart::Text(label.clone()));
            if let Some(content) = attach::image(file) {
                message
                    .parts
                    .push(MessagePart::Attachment(Attachment { label, content }));
            }
        }

        message
    }
}

/// How a message names `file`: its path, as the reader picked it.
fn label(file: &Path) -> String {
    file.to_string_lossy().into_owned()
}

/// The tool calls the app shows: each one's id, by the call it runs for.
#[derive(Resource, Default)]
pub(super) struct Calls {
    ids: HashMap<Entity, String>,
    next: u64,
}

impl Calls {
    /// A new id for the call `call`, kept for its result.
    pub(super) fn start(&mut self, call: Entity) -> String {
        self.next += 1;
        let id = format!("call-{}", self.next);
        self.ids.insert(call, id.clone());
        id
    }

    /// The id of `call`, which a call refused before it ran may not have.
    pub(super) fn end(&mut self, call: Entity) -> String {
        match self.ids.remove(&call) {
            Some(id) => id,
            None => self.start(call),
        }
    }
}

/// Where the last flush read the streams and the notices.
#[derive(Resource, Default)]
pub(super) struct Read {
    deltas: MessageCursor<StreamDelta>,
    notices: MessageCursor<Notice>,
}
fn receive(
    mut spawner: Spawner,
    mut sessions: ResMut<Sessions>,
    mut waiting: ResMut<Waiting>,
    incoming: Option<Res<Incoming>>,
    events: Res<Events>,
    mut commands: Commands,
) {
    let Some(incoming) = incoming else {
        return;
    };
    loop {
        let command = match incoming.0.try_recv() {
            Ok(command) => command,
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                commands.trigger(Exit(ExitReason::Quit));
                break;
            }
        };
        match command {
            Command::Say {
                session,
                model,
                text,
                files,
                before,
                root,
            } => {
                let (agent, note) = match sessions.agent(session) {
                    Some(agent) => (agent, None),
                    None => {
                        let spec = AgentSpec {
                            model: Some(model.clone()),
                            ..AgentSpec::default()
                        };
                        match spawner.spawn(Origin::Root, spec) {
                            Ok(spawned) => {
                                let agent = spawned.id();
                                sessions.bind(session, agent);
                                (agent, (!before.is_empty()).then(|| transcript(&before)))
                            }
                            Err(why) => {
                                let _ = events.0.send(Event::Failed(session, why.to_string()));
                                continue;
                            }
                        }
                    }
                };
                commands.entity(agent).insert(ModelChoice(model));
                // The tools read and run in the session's workspace, and in
                // the app's own directory while it has none.
                match root {
                    Some(root) => commands.entity(agent).insert(WorkDir(root)),
                    None => commands.entity(agent).remove::<WorkDir>(),
                };
                waiting.0.push_back(Pending {
                    session,
                    note,
                    text,
                    files,
                });
            }
            Command::SetModel { session, model } => {
                if let Some(agent) = sessions.agent(session) {
                    commands.entity(agent).insert(ModelChoice(model));
                }
            }
        }
    }
}

/// Sends each waiting prompt to an agent that is connected, and keeps the
/// rest for the frame their agent connects in.
fn deliver(
    sessions: Res<Sessions>,
    mut waiting: ResMut<Waiting>,
    connected: Query<(), (With<ModelChoice>, With<Connection>)>,
    mut commands: Commands,
) {
    let mut kept = VecDeque::new();
    while let Some(pending) = waiting.0.pop_front() {
        match sessions.agent(pending.session) {
            Some(agent) if connected.get(agent).is_ok() => {
                if let Some(note) = &pending.note {
                    commands.trigger(Deliver::new(agent, note.clone(), DeliveryMode::Inform));
                }
                commands.trigger(Deliver {
                    entity: agent,
                    message: pending.message(),
                });
            }
            _ => kept.push_back(pending),
        }
    }
    waiting.0 = kept;
}

/// Writes what the loop streamed this frame.
/// What a new agent is told of the conversation its session already had.
fn transcript(before: &[Turn]) -> String {
    let mut text = String::from("The conversation so far, before this message:");
    for turn in before {
        text.push_str(if turn.user {
            "\n\nUser: "
        } else {
            "\n\nAssistant: "
        });
        text.push_str(&turn.text);
    }
    text
}
