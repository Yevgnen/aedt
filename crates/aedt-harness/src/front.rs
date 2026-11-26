//! The app's front of the harness: it makes an agent per session, hands it
//! what the reader writes, and reports what it does over the channels. The
//! harness's own views mean nothing here, so no terminal or print front runs.

use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use bevy_ecs::{message::MessageCursor, system::SystemParam};
use crossbeam_channel::{Receiver, Sender};
use rig_harness::prelude::*;

use crate::{Command, Event, Turn};

/// The front, made once: the channels it reads and writes.
pub(crate) struct Front {
    commands: Mutex<Option<Receiver<Command>>>,
    events: Sender<Event>,
}

impl Front {
    pub(crate) fn new(commands: Receiver<Command>, events: Sender<Event>) -> Self {
        Self {
            commands: Mutex::new(Some(commands)),
            events,
        }
    }
}

impl Plugin for Front {
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
struct Incoming(Receiver<Command>);

/// What the frame reports, written as it happens.
#[derive(Resource)]
struct Events(Sender<Event>);

/// The agent each session talks to.
#[derive(Resource, Default)]
struct Sessions {
    agents: HashMap<u32, Entity>,
    sessions: HashMap<Entity, u32>,
}

impl Sessions {
    /// The session `agent` answers for, while it is one of ours.
    fn session(&self, agent: Entity) -> Option<u32> {
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
/// its agent must know first, while it is new.
#[derive(Resource, Default)]
struct Waiting(VecDeque<(u32, Option<String>, String)>);

/// The tool calls the app shows: each one's id, by the call it runs for.
#[derive(Resource, Default)]
struct Calls {
    ids: HashMap<Entity, String>,
    next: u64,
}

impl Calls {
    /// A new id for the call `call`, kept for its result.
    fn start(&mut self, call: Entity) -> String {
        self.next += 1;
        let id = format!("call-{}", self.next);
        self.ids.insert(call, id.clone());
        id
    }

    /// The id of `call`, which a call refused before it ran may not have.
    fn end(&mut self, call: Entity) -> String {
        match self.ids.remove(&call) {
            Some(id) => id,
            None => self.start(call),
        }
    }
}

/// Where the last flush read the streams and the notices.
#[derive(Resource, Default)]
struct Read {
    deltas: MessageCursor<StreamDelta>,
    notices: MessageCursor<Notice>,
}

/// What a frame writes to the app: the text streamed since the last write,
/// then whatever the caller adds.
#[derive(SystemParam)]
struct Out<'w, 's> {
    events: Res<'w, Events>,
    sessions: Res<'w, Sessions>,
    deltas: Res<'w, Messages<StreamDelta>>,
    notices: Res<'w, Messages<Notice>>,
    turn_calls: Query<'w, 's, (), With<TurnCall>>,
    read: ResMut<'w, Read>,
}

impl Out<'_, '_> {
    /// Writes the notices and the text that arrived since the last write.
    fn flush(&mut self) {
        let notices: Vec<Notice> = self
            .read
            .notices
            .read(&self.notices)
            .filter(|notice| notice.level == NoticeLevel::Error)
            .cloned()
            .collect();
        for notice in notices {
            let Some(session) = notice.agent.and_then(|agent| self.sessions.session(agent)) else {
                continue;
            };
            self.send(Event::Failed(session, notice.text));
        }

        let deltas: Vec<StreamDelta> = self.read.deltas.read(&self.deltas).cloned().collect();
        for delta in deltas {
            if !self.turn_calls.contains(delta.effect) {
                continue;
            }
            let Some(session) = delta.agent.and_then(|agent| self.sessions.session(agent)) else {
                continue;
            };
            match delta.event {
                StreamEvent::Text { text, .. } => {
                    self.send(Event::Answer(session, text));
                }
                StreamEvent::Reasoning { text, .. } => {
                    self.send(Event::Thinking(session, text));
                }
                _ => {}
            }
        }
    }

    /// Reports `event` to the app.
    fn send(&self, event: Event) {
        let _ = self.events.0.send(event);
    }
}

/// Reads what the app said, makes the agent a session needs, and asks it for
/// a new model when the reader picked one.
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
    while let Ok(command) = incoming.0.try_recv() {
        match command {
            Command::Say {
                session,
                model,
                text,
                before,
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
                waiting.0.push_back((session, note, text));
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
    while let Some((session, note, text)) = waiting.0.pop_front() {
        match sessions.agent(session) {
            Some(agent) if connected.get(agent).is_ok() => {
                if let Some(note) = note {
                    commands.trigger(Deliver::new(agent, note, DeliveryMode::Inform));
                }
                commands.trigger(Deliver::new(agent, text, DeliveryMode::Steer));
            }
            _ => kept.push_back((session, note, text)),
        }
    }
    waiting.0 = kept;
}

/// Writes what the loop streamed this frame.
fn flush(mut out: Out) {
    out.flush();
}

/// A tool call started: its name and arguments go to the app before it runs.
fn tool_started(
    start: On<Add<Started>>,
    calls: Query<(&ToolRequest, &ForAgent)>,
    sessions: Res<Sessions>,
    mut ids: ResMut<Calls>,
    mut out: Out,
) {
    let Ok((request, &ForAgent(agent))) = calls.get(start.entity) else {
        return;
    };
    let Some(session) = sessions.session(agent) else {
        return;
    };
    let id = ids.start(start.entity);
    out.flush();
    out.send(Event::ToolCall {
        session,
        id,
        name: request.call.function.name.to_string(),
        args: request.call.function.arguments_json(),
    });
}

/// A tool call answered, or a turn ended.
fn tool_settled(
    settled: On<Add<Settled>>,
    effects: Query<(&ForAgent, Option<&Outcome>, Option<&TurnOutcome>)>,
    calls: Query<&ToolRequest>,
    started: Query<Has<Started>>,
    sessions: Res<Sessions>,
    mut ids: ResMut<Calls>,
    mut out: Out,
) {
    let Ok((&ForAgent(agent), outcome, ended)) = effects.get(settled.entity) else {
        return;
    };
    let Some(session) = sessions.session(agent) else {
        return;
    };

    if let (Ok(request), Some(outcome)) = (calls.get(settled.entity), outcome) {
        // A call refused before it ran never said what it was: it does now.
        let ran = started.get(settled.entity).is_ok_and(|started| started);
        let id = if ran {
            ids.end(settled.entity)
        } else {
            let id = ids.start(settled.entity);
            out.flush();
            out.send(Event::ToolCall {
                session,
                id: id.clone(),
                name: request.call.function.name.to_string(),
                args: request.call.function.arguments_json(),
            });
            id
        };
        let result = outcome.tool_result();
        out.flush();
        out.send(Event::ToolResult {
            session,
            id,
            name: request.call.function.name.to_string(),
            ok: result.is_success(),
            output: result.output().render(),
        });
    } else if let Some(ended) = ended {
        out.flush();
        match ended {
            TurnOutcome::Answered(_) => out.send(Event::Ended(session)),
            TurnOutcome::Failed(why) => out.send(Event::Failed(session, why.clone())),
            TurnOutcome::Stopped(stop) => {
                out.send(Event::Failed(session, stop.cause().to_string()));
            }
        }
    }
}

/// The agent could not use the model it was given: the prompts waiting for it
/// will never go, and the reader is told why.
fn model_refused(
    added: On<Add<ModelRefused>>,
    refusals: Query<&ModelRefused>,
    sessions: Res<Sessions>,
    mut waiting: ResMut<Waiting>,
    mut out: Out,
) {
    let Ok(refusal) = refusals.get(added.entity) else {
        return;
    };
    let Some(session) = sessions.session(added.entity) else {
        return;
    };
    waiting.0.retain(|(queued, _, _)| *queued != session);
    out.flush();
    out.send(Event::Failed(session, refusal.0.clone()));
}

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
