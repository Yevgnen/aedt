//! The app's state: the sessions and their messages stay behind these
//! functions. A global is per surface, so each window has its own.

use chrono::Local;
use nsh_core::prelude::*;

mod stored;

/// Who wrote a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Author {
    User,
    Assistant,
}

/// One message of a session.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: u32,
    pub author: Author,
    /// What it says, in the order it was written: a reply's thoughts, the
    /// calls it made and the stretches of its answer, as they came.
    pub parts: Vec<Part>,
    /// The files the message carried, in the order they were picked. The model
    /// reads the message's text, which names each file's path; this is what the
    /// transcript draws before the words, a picture as a message of its own.
    pub files: Vec<SharedString>,
    /// Whether the reply's thought shows open at first: the state a reply is
    /// read with, until the reader folds it away.
    pub reasoning_open: bool,
    /// The muted line under the message: when it was sent, and for a reply what
    /// it ran on and how long it took. `None` shows no line at all.
    pub meta: Option<Meta>,
    /// Whether the text is still arriving: a reply's text view then reads it as
    /// one being written, so only the blocks that changed are shown again
    /// instead of the whole document on every token.
    pub streaming: bool,
}

/// One stretch of a message, in the order the reply wrote it.
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    /// A chain of thought that led to what came next.
    Thought(SharedString),
    /// A tool call the reply made.
    Tool(ToolRun),
    /// A stretch of what the reply answered.
    Text(SharedString),
}

impl Message {
    /// What it answered, as it stands: the text of every stretch it wrote, in
    /// order. Empty for a reply that only thought or called tools.
    pub fn body(&self) -> String {
        self.parts
            .iter()
            .filter_map(|part| match part {
                Part::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }
}

/// One tool call a reply made: what the model asked, and what came of it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolRun {
    /// The call's id, which its result answers.
    pub id: SharedString,
    pub name: SharedString,
    /// The arguments as the model sent them: a JSON object.
    pub arguments: SharedString,
    /// What the tool returned: empty while it runs.
    pub output: SharedString,
    /// Whether the call failed (or never ran), `output` saying why then.
    pub failed: bool,
    /// Whether its result is still to come.
    pub running: bool,
    /// Whether its output shows open at first, until the reader folds it
    /// away.
    pub open: bool,
    /// The files it changed, for a call that edits them: each with the lines
    /// it touched and the change itself, which the card shows in place of the
    /// output.
    pub changes: Vec<ToolChange>,
}

/// One file a tool call changed.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolChange {
    /// The file, as the tool named it.
    pub path: SharedString,
    /// Lines it added.
    pub added: usize,
    /// Lines it removed.
    pub removed: usize,
    /// The change as a unified diff: its lines, each prefixed by the sign that
    /// says what it does (` `, `-` or `+`).
    pub diff: SharedString,
}

/// What a message's muted line says.
#[derive(Clone, Debug, PartialEq)]
pub struct Meta {
    /// When it was sent, as the app writes it (`11/6 21:12`).
    pub time: SharedString,
    /// What a reply adds of itself: the model it ran on and how long it took.
    /// A message the reader wrote has none.
    pub detail: Option<SharedString>,
}

/// What an untitled session shows until its first message names it.
pub const UNTITLED: &str = "New chat";

/// One conversation in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: u32,
    pub title: SharedString,
    pub messages: Vec<Message>,
    /// The folder the conversation works in, while it works in one: its
    /// agent's tools read and run there. `None` leaves it in the app's own
    /// directory, under `Ungrouped` in the sidebar.
    pub workspace: Option<u32>,
    /// Whether the reader put it away: it leaves the sidebar with its
    /// messages kept.
    pub archived: bool,
}

/// A folder the conversations of a group work in.
#[derive(Clone, Debug, PartialEq)]
pub struct Workspace {
    pub id: u32,
    /// Where it is, as the reader picked it.
    pub path: SharedString,
    /// What it is called: the last part of its path.
    pub name: SharedString,
}

/// A workspace's conversations, or the ones held in no workspace, listed
/// last under the sidebar's `Ungrouped`.
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub workspace: Option<Workspace>,
    pub sessions: Vec<SessionSummary>,
}

impl Session {
    /// What the sidebar and the header show: an untitled session keeps the
    /// placeholder until its first message names it.
    pub fn display_title(&self) -> SharedString {
        if self.title.is_empty() {
            UNTITLED.into()
        } else {
            self.title.clone()
        }
    }

    /// What a row and the header show of it, without its messages.
    fn summary(&self) -> SessionSummary {
        SessionSummary {
            id: self.id,
            title: self.display_title(),
        }
    }
}

/// A session as the sidebar and the header show it: its identity and title,
/// never its messages, so reading one does not copy a whole conversation.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionSummary {
    pub id: u32,
    pub title: SharedString,
}

#[derive(Clone)]
struct State {
    sessions: Vec<Session>,
    workspaces: Vec<Workspace>,
    selected: u32,
    next_session_id: u32,
    next_workspace_id: u32,
    next_message_id: u32,
}

impl Default for State {
    fn default() -> Self {
        if let Some(state) = stored::load() {
            return state;
        }

        Self {
            sessions: vec![Session {
                id: 1,
                title: "".into(),
                workspace: None,
                archived: false,
                messages: Vec::new(),
            }],
            workspaces: Vec::new(),
            selected: 1,
            next_session_id: 2,
            next_workspace_id: 1,
            next_message_id: 1,
        }
    }
}

/// The sessions a model is writing a title for: one title types in at a time,
/// and another ask waits for it.
#[derive(Clone, Default)]
struct Writing(Vec<u32>);

/// The clock time a message shows, in the machine's own time zone: `11/6
/// 21:12` (month and day without a leading zero).
fn now_label() -> SharedString {
    Local::now().format("%-m/%-d %H:%M").to_string().into()
}

fn state() -> Signal<State> {
    global::<State>()
}

fn writing() -> Signal<Writing> {
    global::<Writing>()
}

/// The state, kept for the next run.
fn keep() {
    state().with(stored::save);
}

/// Session `id` of `state`.
fn session(state: &State, id: u32) -> Option<&Session> {
    state.sessions.iter().find(|session| session.id == id)
}

/// Session `id` of `state`, to change.
fn session_mut(state: &mut State, id: u32) -> Option<&mut Session> {
    state.sessions.iter_mut().find(|session| session.id == id)
}

/// What the session `id` shows, or none once it is gone (tracked where it is
/// read).
pub fn session_summary(id: u32) -> Option<SessionSummary> {
    state().with(|workspace| session(workspace, id).map(Session::summary))
}

/// The session the conversation pane shows (tracked where it is read).
pub fn selected_session_id() -> u32 {
    state().with(|workspace| workspace.selected)
}

/// Shows `id` in the conversation pane; an unknown id changes nothing.
pub fn select_session(id: u32) {
    let (current, exists) = state().with(|workspace| {
        (
            workspace.selected,
            workspace.sessions.iter().any(|session| session.id == id),
        )
    });
    if current == id || !exists {
        return;
    }
    state().update(|workspace| workspace.selected = id);
    keep();
}

/// Adds an empty session above the others and shows it, in `workspace`; when
/// the session on show is already an empty one in that workspace, it stays
/// the one shown rather than another empty one joining it.
pub fn create_session_in(workspace: Option<u32>) {
    let already_new = state().with(|state| {
        session(state, state.selected)
            .is_some_and(|session| session.workspace == workspace && session.messages.is_empty())
    });
    if already_new {
        return;
    }

    let id = state().with(|state| state.next_session_id);
    state().update(|state| {
        state.next_session_id += 1;
        state.sessions.insert(
            0,
            Session {
                id,
                title: "".into(),
                messages: Vec::new(),
                workspace,
                archived: false,
            },
        );
        state.selected = id;
    });
    keep();
}

/// Takes the session `id` out of the store for good, showing another in its
/// place while it was the one on show.
pub fn delete_session(id: u32) {
    state().update(|state| {
        let Some(index) = state.sessions.iter().position(|session| session.id == id) else {
            return;
        };
        leave(state, index, false);
    });
    keep();
}

/// Puts the session `id` away: it leaves the sidebar with its messages kept,
/// and another shows while it was the one on show.
pub fn archive_session(id: u32) {
    state().update(|state| {
        let Some(index) = state.sessions.iter().position(|session| session.id == id) else {
            return;
        };
        leave(state, index, true);
    });
    keep();
}

/// Takes the session at `index` out of the sidebar, archived rather than
/// removed when `archived` holds, and shows what takes its place while it was
/// the one on show: the next session of its group, else the one before, else
/// the first one left; with none left, an empty one joins its workspace.
fn leave(state: &mut State, index: usize, archived: bool) {
    let session = &state.sessions[index];
    let (id, workspace) = (session.id, session.workspace);
    if archived {
        state.sessions[index].archived = true;
    } else {
        state.sessions.remove(index);
    }
    if state.selected != id {
        return;
    }

    let shown = |session: &Session| !session.archived && session.id != id;
    let next = state.sessions[index..]
        .iter()
        .find(|session| shown(session) && session.workspace == workspace)
        .or_else(|| {
            state.sessions[..index]
                .iter()
                .rev()
                .find(|session| shown(session) && session.workspace == workspace)
        })
        .or_else(|| state.sessions.iter().find(|session| shown(session)))
        .map(|session| session.id);

    match next {
        Some(id) => state.selected = id,
        None => {
            let id = state.next_session_id;
            state.next_session_id += 1;
            state.sessions.insert(
                0,
                Session {
                    id,
                    title: "".into(),
                    messages: Vec::new(),
                    workspace,
                    archived: false,
                },
            );
            state.selected = id;
        }
    }
}

/// Adds the folder at `path` as a workspace, or gives the one already there.
/// `None` for a path that names no folder.
pub fn add_workspace(path: &str) -> Option<u32> {
    let path = path.trim_end_matches('/');
    if path.is_empty() {
        return None;
    }
    let mut added = None;
    state().update(|state| {
        if let Some(workspace) = state
            .workspaces
            .iter()
            .find(|workspace| workspace.path.as_str() == path)
        {
            added = Some(workspace.id);
            return;
        }
        let id = state.next_workspace_id;
        state.next_workspace_id += 1;
        state.workspaces.push(Workspace {
            id,
            path: path.to_owned().into(),
            name: folder_name(path),
        });
        added = Some(id);
    });
    keep();
    added
}

/// Takes the workspace `id` out of the sidebar, its folder left where it is:
/// its conversations join the ones held in none.
pub fn remove_workspace(id: u32) {
    state().update(|state| {
        state.workspaces.retain(|workspace| workspace.id != id);
        for session in &mut state.sessions {
            if session.workspace == Some(id) {
                session.workspace = None;
            }
        }
    });
    keep();
}

/// The conversations, grouped: each workspace's under it, the ones held in
/// none last, and the archived ones never (tracked where it is read).
pub fn groups() -> Vec<Group> {
    state().with(|state| {
        let mut groups: Vec<Group> = state
            .workspaces
            .iter()
            .map(|workspace| Group {
                workspace: Some(workspace.clone()),
                sessions: Vec::new(),
            })
            .collect();
        let mut ungrouped = Group {
            workspace: None,
            sessions: Vec::new(),
        };
        for session in state.sessions.iter().filter(|session| !session.archived) {
            let summary = session.summary();
            let group = session.workspace.and_then(|id| {
                groups
                    .iter_mut()
                    .find(|group| group.workspace.as_ref().is_some_and(|held| held.id == id))
            });
            match group {
                Some(group) => group.sessions.push(summary),
                None => ungrouped.sessions.push(summary),
            }
        }
        groups.push(ungrouped);
        groups
    })
}

/// Where `session` works: its workspace's folder, while it has one.
pub fn session_path(session_id: u32) -> Option<SharedString> {
    state().with(|state| {
        let id = session(state, session_id)?.workspace?;
        state
            .workspaces
            .iter()
            .find(|workspace| workspace.id == id)
            .map(|workspace| workspace.path.clone())
    })
}

/// What a folder is called: the last part of its path.
fn folder_name(path: &str) -> SharedString {
    std::path::Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_owned()
        .into()
}

/// How many messages a session has (tracked where it is read).
pub fn message_count(session_id: u32) -> usize {
    state().with(|workspace| {
        session(workspace, session_id).map_or(0, |session| session.messages.len())
    })
}

/// The messages' ids, in order (tracked where it is read).
pub fn message_ids(session_id: u32) -> Vec<u32> {
    state().with(|workspace| {
        session(workspace, session_id)
            .map(|session| session.messages.iter().map(|message| message.id).collect())
            .unwrap_or_default()
    })
}

/// The message at `index`, or none past the end (tracked where it is read).
pub fn message_at(session_id: u32, index: usize) -> Option<Message> {
    state().with(|workspace| {
        session(workspace, session_id)
            .and_then(|session| session.messages.get(index))
            .cloned()
    })
}

/// The last user message among the rows `first..=last`: the one the rail marks
/// as the reader's current place.
pub fn last_user_index(session_id: u32, first: usize, last: usize) -> Option<usize> {
    state().with(|workspace| {
        let session = session(workspace, session_id)?;
        let last = last.min(session.messages.len().checked_sub(1)?);
        (first..=last)
            .rev()
            .find(|index| session.messages[*index].author == Author::User)
    })
}

/// How many user messages the session has: one rail tick each (tracked where it
/// is read).
pub fn user_message_count(session_id: u32) -> usize {
    state().with(|workspace| {
        session(workspace, session_id).map_or(0, |session| {
            session
                .messages
                .iter()
                .filter(|message| message.author == Author::User)
                .count()
        })
    })
}

/// The indices of the session's user messages, in order: one tick of the rail
/// each.
pub fn user_message_indices(session_id: u32) -> Vec<usize> {
    state().with(|workspace| {
        session(workspace, session_id)
            .map(|session| {
                session
                    .messages
                    .iter()
                    .enumerate()
                    .filter(|(_, message)| message.author == Author::User)
                    .map(|(index, _)| index)
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// The reply that follows the user message at `index`: the assistant messages
/// up to the next question.
pub fn reply_at(session_id: u32, index: usize) -> Option<Message> {
    state().with(|workspace| {
        let session = session(workspace, session_id)?;
        session
            .messages
            .iter()
            .skip(index + 1)
            .take_while(|message| message.author == Author::Assistant)
            .next()
            .cloned()
    })
}

/// What sending a message left to answer: the reply waiting for it, and the
/// conversation the model reads.
pub struct Sent {
    pub reply_id: u32,
    pub history: Vec<Message>,
}

/// Sends `text`: appends the user's message and an empty reply. A blank message
/// sends nothing, and returns nothing to answer.
/// Sends `text`, the user's message with the files it carries, and answers it
/// with an empty reply to fill: what the model gets to read is every message
/// before that reply.
pub fn send(session_id: u32, text: &str, files: &[SharedString]) -> Option<Sent> {
    let text = text.trim();
    if text.is_empty() && files.is_empty() {
        return None;
    }

    let mut started = None;
    state().update(|workspace| started = append_sent(workspace, session_id, text, files));
    if started.is_some() {
        keep();
    }

    started.map(|(reply_id, history)| Sent { reply_id, history })
}

/// The user's message, the reply left empty, and what the model gets to read:
/// every message before that reply.
fn append_sent(
    state: &mut State,
    session_id: u32,
    text: &str,
    files: &[SharedString],
) -> Option<(u32, Vec<Message>)> {
    let user_id = state.next_message_id;
    let reply_id = user_id + 1;
    state.next_message_id += 2;

    let sent_at = now_label();
    let session = session_mut(state, session_id)?;
    if session.title.is_empty() {
        session.title = title_from(text);
    }
    session.messages.push(Message {
        id: user_id,
        author: Author::User,
        parts: vec![Part::Text(text.to_owned().into())],
        files: files.to_vec(),
        reasoning_open: false,
        meta: Some(Meta {
            time: sent_at.clone(),
            detail: None,
        }),
        streaming: false,
    });
    let history = session.messages.clone();
    session.messages.push(Message {
        id: reply_id,
        author: Author::Assistant,
        parts: Vec::new(),
        files: Vec::new(),
        reasoning_open: false,
        meta: Some(Meta {
            time: sent_at,
            detail: None,
        }),
        streaming: true,
    });

    Some((reply_id, history))
}

/// Appends what arrived of the reply still streaming in `session`: its
/// thinking first, then its answer, each where the reply's order puts it.
pub fn append_stream(session_id: u32, thinking: &str, answer: &str) {
    if thinking.is_empty() && answer.is_empty() {
        return;
    }
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        if !thinking.is_empty() {
            match message.parts.last_mut() {
                Some(Part::Thought(thought)) => {
                    let mut reasoned = thought.to_string();
                    reasoned.push_str(thinking);
                    *thought = reasoned.into();
                }
                _ => message
                    .parts
                    .push(Part::Thought(thinking.to_owned().into())),
            }
        }
        if !answer.is_empty() {
            match message.parts.last_mut() {
                Some(Part::Text(text)) => {
                    let mut body = text.to_string();
                    body.push_str(answer);
                    *text = body.into();
                }
                _ => message.parts.push(Part::Text(answer.to_owned().into())),
            }
        }
    });
}

/// Records that the reply streaming in `session` asked for a tool call: `id`,
/// its name and the arguments the model sent, shown while the tool runs.
///
/// The call takes its place in the reply as it came: what the model thought
/// next is a thought of its own, after it.
pub fn tool_started(session_id: u32, id: &str, name: &str, arguments: &str) {
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        message.parts.push(Part::Tool(ToolRun {
            id: id.to_owned().into(),
            name: name.to_owned().into(),
            arguments: arguments.to_owned().into(),
            output: SharedString::default(),
            failed: false,
            running: true,
            open: false,
            changes: Vec::new(),
        }));
    });
}

/// Records what a tool call of the reply streaming in `session` returned: the
/// answer of the call `id`, whether it failed, and the files it changed.
pub fn tool_finished(
    session_id: u32,
    id: &str,
    output: &str,
    failed: bool,
    changes: Vec<ToolChange>,
) {
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        let Some(tool) = message.parts.iter_mut().find_map(|part| match part {
            Part::Tool(tool) if tool.id.as_str() == id => Some(tool),
            _ => None,
        }) else {
            return;
        };
        tool.output = output.to_owned().into();
        tool.failed = failed;
        tool.running = false;
        tool.changes = changes;
    });
}

/// Closes the reply streaming in `session`: its meta line gains the model and
/// how long the answer took, after the time it was sent.
pub fn finish_stream(session_id: u32, model: &str, seconds: f32) {
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        let time = message
            .meta
            .as_ref()
            .map(|meta| meta.time.clone())
            .unwrap_or_default();
        message.streaming = false;
        message.meta = Some(Meta {
            time,
            detail: Some(format!("{model} · {seconds:.1} s").into()),
        });
    });
    keep();
}

/// Shows what went wrong in place of the answer of the reply streaming in
/// `session`.
pub fn fail_stream(session_id: u32, error: &str) {
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        message
            .parts
            .push(Part::Text(format!("Request failed: {error}").into()));
        message.streaming = false;
    });
    keep();
}

/// Shows what went wrong in place of every answer still streaming: the
/// agent itself could not run.
pub fn fail_streaming(error: &str) {
    state().update(|workspace| {
        for session in &mut workspace.sessions {
            for message in &mut session.messages {
                if message.streaming {
                    message
                        .parts
                        .push(Part::Text(format!("Request failed: {error}").into()));
                    message.streaming = false;
                }
            }
        }
    });
    keep();
}

/// The thought slots of `session` that show open at first, each by its
/// message and its place in it: a reply's thought opens under its message's
/// id.
pub fn open_thoughts(session_id: u32) -> Vec<(u32, usize)> {
    state().with(|workspace| {
        session(workspace, session_id)
            .map(|session| {
                session
                    .messages
                    .iter()
                    .filter(|message| message.reasoning_open)
                    .flat_map(|message| {
                        message
                            .parts
                            .iter()
                            .enumerate()
                            .filter(|(_, part)| matches!(part, Part::Thought(_)))
                            .map(move |(at, _)| (message.id, at))
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// The tool calls of `session` whose output shows open at first.
pub fn open_calls(session_id: u32) -> Vec<SharedString> {
    state().with(|workspace| {
        session(workspace, session_id)
            .map(|session| {
                session
                    .messages
                    .iter()
                    .flat_map(|message| message.parts.iter())
                    .filter_map(|part| match part {
                        Part::Tool(tool) if tool.open => Some(tool.id.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// The reply still arriving in `session`.
fn streaming(state: &mut State, session_id: u32) -> Option<&mut Message> {
    session_mut(state, session_id)?
        .messages
        .iter_mut()
        .rev()
        .find(|message| message.streaming)
}

/// Starts writing session `session_id`'s title: what it shows stays until the
/// model's first words type in (tracked where it is read).
pub fn title_started(session_id: u32) {
    writing().update(|writing| {
        if !writing.0.contains(&session_id) {
            writing.0.push(session_id);
        }
    });
}

/// Shows `title` as so much of session `session_id`'s, while its model types it
/// in; false once the session is gone, so the typing stops.
pub fn title_typed(session_id: u32, title: &str) -> bool {
    let mut shown = false;
    state().update(|state| {
        if let Some(session) = session_mut(state, session_id) {
            session.title = title.to_owned().into();
            shown = true;
        }
    });

    shown
}

/// Keeps `title` as session `session_id`'s, its model done with it.
pub fn title_finished(session_id: u32, title: &str) {
    title_typed(session_id, title);
    title_failed(session_id);
    keep();
}

/// Stops writing session `session_id`'s title: it keeps what it shows.
pub fn title_failed(session_id: u32) {
    writing().update(|writing| writing.0.retain(|id| *id != session_id));
}

/// Whether a model is writing session `session_id`'s title (tracked where it is
/// read).
pub fn title_writing(session_id: u32) -> bool {
    writing().with(|writing| writing.0.contains(&session_id))
}

/// The first `max` characters of `text`, on one line.
fn preview(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = flat.chars();
    let head: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

/// The sidebar title a first message gives its session.
fn title_from(text: &str) -> SharedString {
    preview(text, 18).into()
}
