//! What the harness reports of a reply: the text it streams, its tool calls and
//! their results, and what an agent refuses.

use bevy_ecs::system::SystemParam;
use rig_harness::prelude::*;
use rig_harness::rig_core::tool::FilesChanged;

use crate::{ChangedFile, Event};

use super::{Calls, Events, Read, Sessions, Waiting};

/// What a frame writes to the app: the text streamed since the last write,
/// then whatever the caller adds.
#[derive(SystemParam)]
pub(super) struct Out<'w, 's> {
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
/// a new model when the reader picked one. The harness ends when the app lets
/// go of its handle: nothing will be said to it again.
pub(super) fn flush(mut out: Out) {
    out.flush();
}

/// A tool call started: its name and arguments go to the app before it runs.
pub(super) fn tool_started(
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

/// The effect a settled call or turn answered: whose it was, what it
/// answered, and what it published beside the answer.
type Effects<'w, 's> = Query<
    'w,
    's,
    (
        &'static ForAgent,
        Option<&'static Outcome>,
        Option<&'static TurnOutcome>,
        Option<&'static Published>,
    ),
>;

/// A tool call answered, or a turn ended.
pub(super) fn tool_settled(
    settled: On<Add<Settled>>,
    effects: Effects,
    calls: Query<&ToolRequest>,
    started: Query<Has<Started>>,
    sessions: Res<Sessions>,
    mut ids: ResMut<Calls>,
    mut out: Out,
) {
    let Ok((&ForAgent(agent), outcome, ended, published)) = effects.get(settled.entity) else {
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
            changes: changes(published),
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

/// The files a call changed, as the app shows them: what a call that edits
/// published ([`FilesChanged`]), with each file's whole diff.
fn changes(published: Option<&Published>) -> Vec<ChangedFile> {
    published
        .and_then(|published| published.0.get::<FilesChanged>().ok().flatten())
        .map(|changed| {
            changed
                .0
                .into_iter()
                .map(|change| {
                    let diff = change.unified();

                    ChangedFile {
                        path: change.path,
                        added: change.added,
                        removed: change.removed,
                        diff,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The agent could not use the model it was given: the prompts waiting for it
/// will never go, and the reader is told why.
pub(super) fn model_refused(
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
