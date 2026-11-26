//! The app's state: the sessions and their messages stay behind these
//! functions. A global is per surface, so each window has its own.

use chrono::Local;
use nsh_core::prelude::*;

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
    pub body: SharedString,
    /// The chains of thought the reply reasoned with, in order and shown above
    /// it: empty for a message that did not think, one entry per thought.
    pub reasoning: Vec<SharedString>,
    /// Whether the reply's thought shows open at first: the state a reply is
    /// read with, until the reader folds it away.
    pub reasoning_open: bool,
    /// The tool calls the reply made, in the order it made them.
    pub tools: Vec<ToolRun>,
    /// The muted line under the message: when it was sent, and for a reply what
    /// it ran on and how long it took. `None` shows no line at all.
    pub meta: Option<Meta>,
    /// Whether the text is still arriving: a reply's text view then reads it as
    /// one being written, so only the blocks that changed are shown again
    /// instead of the whole document on every token.
    pub streaming: bool,
}

/// One tool call a reply made: what the model asked, and what came of it.
#[derive(Clone, Debug, PartialEq)]
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
        let mut next_message_id = 1;
        let first = Session {
            id: 1,
            title: "Laying out the session sidebar".into(),
            workspace: None,
            messages: vec![
                message(
                    &mut next_message_id,
                    Author::User,
                    "Start with a skeleton: the session list on the left, the chat on the right.",
                    "11/25 10:12",
                    None,
                ),
                thought_open(with_thoughts(
                    message(
                        &mut next_message_id,
                        Author::Assistant,
                        "Pin three things down:\n\n1. The session is the only domain object, and messages belong to it;\n2. The list uses `virtual_list`, and no row keeps state;\n3. The composer's draft belongs to the page, not to the store.\n\n```rust\nlet draft = signal(String::new());\n```",
                        "11/25 10:12",
                        Some("deepseek-flash · 0.3 s"),
                    ),
                    &["The questions after this one are about state, not layout. \
                       Separate the two: decide where state lives first, then pick \
                       the components that read it."],
                )),
                message(
                    &mut next_message_id,
                    Author::User,
                    "How should mock data echo while I type?",
                    "11/25 10:14",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "The draft is the page's signal: the input binds to it, and so does the echo bar. A keystroke reruns only those two bindings; the list and the sidebar stay put.",
                    "11/25 10:14",
                    Some("deepseek-flash · 0.2 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "Why can't a row keep its state in a signal while the message list scrolls?",
                    "11/25 10:17",
                    None,
                ),
                thought_open(with_thoughts(
                    message(
                        &mut next_message_id,
                        Author::Assistant,
                        "Because `virtual_list` reuses its cells: one cell shows different rows in turn. A row's signal travels with the cell to the next row, and states bleed. Keep each row's state in the data, keyed by message id; the cell reads only the one at its position.",
                        "11/25 10:17",
                        Some("deepseek-flash · 0.3 s"),
                    ),
                    &[
                        "The symptom is a value that follows the scroll instead of \
                         the row: that is state living in the cell, not a paint bug.",
                        "`virtual_list` recycles cells, so anything a cell keeps outlives \
                         the row it showed. Move the state into the data, keyed by \
                         message id, and the cell only reads it.",
                    ],
                )),
                message(
                    &mut next_message_id,
                    Author::User,
                    "Why is the draft gone after switching sessions?",
                    "11/25 10:21",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "The pane is rebuilt by `dynamic` per id, so the draft's signal and the scroll position belong to that build and are released with it. To keep them, store drafts by session id and restore one when its pane opens.",
                    "11/25 10:21",
                    Some("deepseek-flash · 0.2 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "How is the sidebar's selected state drawn?",
                    "11/25 10:24",
                    None,
                ),
                with_thoughts(
                    message(
                        &mut next_message_id,
                        Author::Assistant,
                        "Each row compares its session id with the selected one: the selected row takes the `accent` fill and `foreground` text, and a lighter wash on hover. No left bar, and no bold text standing in for selection.",
                        "11/25 10:24",
                        Some("deepseek-flash · 0.2 s"),
                    ),
                    &[
                        "Selection is a comparison, not a flag a row stores: the id \
                       the row shows is enough for it to know whether it is the one.",
                    ],
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "Can a code block in a long reply burst the message column?",
                    "11/25 10:28",
                    None,
                ),
                with_tools(
                    message(
                        &mut next_message_id,
                        Author::Assistant,
                        "An assistant message renders Markdown with `text_view`, and a code block scrolls sideways in its own area, so it never bursts the column; a user message stays plain text, at most 560pt wide.\n\n```rust\nfn main() {\n    println!(\"Aedt\");\n}\n```\n\nThis long body is here to push the list into scrolling.",
                        "11/25 10:28",
                        Some("deepseek-flash · 0.4 s"),
                    ),
                    vec![
                        tool(
                            "call-1",
                            "read",
                            r#"{"path":"Cargo.toml"}"#,
                            "[package]\nname = \"aedt\"\nversion = \"0.1.0\"\nedition = \"2024\"",
                            false,
                        ),
                        tool(
                            "call-2",
                            "shell",
                            r#"{"command":"wc -l src/pages/chat.rs"}"#,
                            "747 src/pages/chat.rs",
                            true,
                        ),
                    ],
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "Does dark mode need extra color work?",
                    "11/25 10:33",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "No. Every color comes from `theme()`, and `card`, `border` and `muted` each carry a light and a dark value; pages write no hex, and a missing role goes into the theme first.",
                    "11/25 10:33",
                    Some("deepseek-flash · 0.2 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "What else should I check before shipping?",
                    "11/25 10:37",
                    None,
                ),
                with_thoughts(
                    message(
                        &mut next_message_id,
                        Author::Assistant,
                        "That the keyboard can finish the main flow, focus is always visible, the empty, loading and failed states all exist, both light and dark have been seen, and the smallest window does not break the layout. Then run a real window on the target platform, not just screenshots.",
                        "11/25 10:37",
                        Some("deepseek-flash · 0.2 s"),
                    ),
                    &[
                        "Most of these are states nobody clicks while testing: empty, \
                         loading, failed, and the smallest window.",
                        "A screenshot cannot show focus, scrolling or a window that \
                         fails to open, so the last check has to be a real run.",
                    ],
                ),
            ],
        };
        let second = Session {
            id: 2,
            title: "Debugging an nsh window that won't start".into(),
            workspace: None,
            messages: vec![
                message(
                    &mut next_message_id,
                    Author::User,
                    "The window is empty after `cargo run`.",
                    "11/25 09:41",
                    None,
                ),
                with_tools(
                    message(
                        &mut next_message_id,
                        Author::Assistant,
                        "First check that the root fills the window: give it `.flex_1()` and the theme's background, or nothing gets drawn behind the window.",
                        "11/25 09:41",
                        Some("deepseek-flash · 0.2 s"),
                    ),
                    vec![tool(
                        "call-3",
                        "shell",
                        r#"{"command":"ls -la src"}"#,
                        "total 64\ndrwxr-xr-x   5 maximin  staff    160 11/25 09:38 .\n-rw-r--r--   1 maximin  staff     86 11/25 09:38 lib.rs\n-rw-r--r--   1 maximin  staff    188 11/25 09:38 llm.rs\ndrwxr-xr-x   4 maximin  staff    128 11/25 09:38 pages",
                        true,
                    )],
                ),
            ],
        };
        let third = Session {
            id: 3,
            title: "Timeouts for tool calls".into(),
            workspace: None,
            messages: vec![with_tools(
                with_thoughts(
                    message(
                        &mut next_message_id,
                        Author::Assistant,
                        "Every tool call needs a timeout, and a timeout must reach the layer above apart from a cancellation: the caller retries one and reports the other.\n\n```rust\nlet timeout = Duration::from_secs(30);\n```",
                        "11/25 08:15",
                        Some("deepseek-flash · 0.1 s"),
                    ),
                    &[
                        "The two look the same from below: both stop the wait. \
                         Only the layer above can tell whether to try again.",
                        "So the call has to answer with why it stopped, not with \
                         whether it did.",
                    ],
                ),
                vec![
                    ToolRun {
                        failed: true,
                        ..tool(
                            "call-4",
                            "shell",
                            r#"{"command":"rg timeouts src"}"#,
                            "command not found: rg",
                            true,
                        )
                    },
                    tool(
                        "call-5",
                        "read",
                        r#"{"path":"crates/aedt-harness/src/front.rs"}"#,
                        "// The harness's own views mean nothing here, so no terminal or print front runs.",
                        false,
                    ),
                ],
            )],
        };

        Self {
            sessions: vec![first, second, third],
            workspaces: Vec::new(),
            selected: 1,
            next_session_id: 4,
            next_workspace_id: 1,
            next_message_id,
        }
    }
}

fn message(
    next_id: &mut u32,
    author: Author,
    body: impl Into<SharedString>,
    time: &'static str,
    detail: Option<&'static str>,
) -> Message {
    let message = Message {
        id: *next_id,
        author,
        body: body.into(),
        reasoning: Vec::new(),
        reasoning_open: false,
        tools: Vec::new(),
        meta: Some(Meta {
            time: time.into(),
            detail: detail.map(SharedString::from),
        }),
        streaming: false,
    };
    *next_id += 1;
    message
}

/// `message` with the chains of thought it reasoned with, in order.
fn with_thoughts(message: Message, thoughts: &[&'static str]) -> Message {
    Message {
        reasoning: thoughts.iter().map(|thought| (*thought).into()).collect(),
        ..message
    }
}

/// `message` with its thought shown open at first.
fn thought_open(message: Message) -> Message {
    Message {
        reasoning_open: true,
        ..message
    }
}

/// `message` with the tool calls it made, in order.
fn with_tools(message: Message, tools: Vec<ToolRun>) -> Message {
    Message { tools, ..message }
}

/// One tool call a reply made, done and read: `open` says whether its output
/// shows at first.
fn tool(
    id: &'static str,
    name: &'static str,
    arguments: &'static str,
    output: &'static str,
    open: bool,
) -> ToolRun {
    ToolRun {
        id: id.into(),
        name: name.into(),
        arguments: arguments.into(),
        output: output.into(),
        failed: false,
        running: false,
        open,
    }
}

/// The clock time a message shows, in the machine's own time zone: `11/6
/// 21:12` (month and day without a leading zero).
fn now_label() -> SharedString {
    Local::now().format("%-m/%-d %H:%M").to_string().into()
}

fn state() -> Signal<State> {
    global::<State>()
}

/// Session `id` of `state`.
fn session(state: &State, id: u32) -> Option<&Session> {
    state.sessions.iter().find(|session| session.id == id)
}

/// Session `id` of `state`, to change.
fn session_mut(state: &mut State, id: u32) -> Option<&mut Session> {
    state.sessions.iter_mut().find(|session| session.id == id)
}

/// How many sessions there are (tracked where it is read).
pub fn session_count() -> usize {
    state().with(|workspace| workspace.sessions.len())
}

/// The sessions' ids, in order (tracked where it is read).
pub fn session_ids() -> Vec<u32> {
    state().with(|workspace| {
        workspace
            .sessions
            .iter()
            .map(|session| session.id)
            .collect()
    })
}

/// What the session at `index` shows, or none past the end (tracked where it is
/// read).
pub fn session_summary_at(index: usize) -> Option<SessionSummary> {
    state().with(|workspace| workspace.sessions.get(index).map(Session::summary))
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
}

/// Adds an empty session above the others and shows it, in the workspace the
/// session on show works in.
pub fn create_session() {
    let workspace =
        state().with(|state| session(state, state.selected).and_then(|session| session.workspace));
    create_session_in(workspace);
}

/// Adds an empty session above the others and shows it, in `workspace`.
pub fn create_session_in(workspace: Option<u32>) {
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
            },
        );
        state.selected = id;
    });
}

/// The workspaces, in the order they were added (tracked where it is read).
pub fn workspaces() -> Vec<Workspace> {
    state().with(|state| state.workspaces.clone())
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
    added
}

/// The conversations, grouped: each workspace's under it, the ones held in
/// none last (tracked where it is read).
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
        for session in &state.sessions {
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
pub fn send(session_id: u32, text: &str) -> Option<Sent> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    let mut started = None;
    state().update(|workspace| started = append_sent(workspace, session_id, text));

    started.map(|(reply_id, history)| Sent { reply_id, history })
}

/// The user's message, the reply left empty, and what the model gets to read:
/// every message before that reply.
fn append_sent(state: &mut State, session_id: u32, text: &str) -> Option<(u32, Vec<Message>)> {
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
        body: text.to_owned().into(),
        reasoning: Vec::new(),
        reasoning_open: false,
        tools: Vec::new(),
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
        body: SharedString::default(),
        reasoning: Vec::new(),
        reasoning_open: false,
        tools: Vec::new(),
        meta: Some(Meta {
            time: sent_at,
            detail: None,
        }),
        streaming: true,
    });

    Some((reply_id, history))
}

/// Appends what arrived of the reply still streaming in `session`: its
/// thinking first, then its answer.
pub fn append_stream(session_id: u32, thinking: &str, answer: &str) {
    if thinking.is_empty() && answer.is_empty() {
        return;
    }
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        if !thinking.is_empty() {
            match message.reasoning.last_mut() {
                Some(thought) => {
                    let mut reasoned = thought.to_string();
                    reasoned.push_str(thinking);
                    *thought = reasoned.into();
                }
                None => message.reasoning.push(thinking.to_owned().into()),
            }
        }
        if !answer.is_empty() {
            let mut body = message.body.to_string();
            body.push_str(answer);
            message.body = body.into();
        }
    });
}

/// Records that the reply streaming in `session` asked for a tool call: `id`,
/// its name and the arguments the model sent, shown while the tool runs.
///
/// A call ends the thought that led to it: what the model thinks while it runs,
/// or after, is a thought of its own.
pub fn tool_started(session_id: u32, id: &str, name: &str, arguments: &str) {
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        message.tools.push(ToolRun {
            id: id.to_owned().into(),
            name: name.to_owned().into(),
            arguments: arguments.to_owned().into(),
            output: SharedString::default(),
            failed: false,
            running: true,
            open: false,
        });
        message.reasoning.push(SharedString::default());
    });
}

/// Records what a tool call of the reply streaming in `session` returned: the
/// answer of the call `id`, and whether it failed.
pub fn tool_finished(session_id: u32, id: &str, output: &str, failed: bool) {
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        let Some(tool) = message.tools.iter_mut().find(|tool| tool.id.as_str() == id) else {
            return;
        };
        tool.output = output.to_owned().into();
        tool.failed = failed;
        tool.running = false;
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
}

/// Shows what went wrong in place of the answer of the reply streaming in
/// `session`.
pub fn fail_stream(session_id: u32, error: &str) {
    state().update(|workspace| {
        let Some(message) = streaming(workspace, session_id) else {
            return;
        };
        message.body = format!("Request failed: {error}").into();
        message.streaming = false;
    });
}

/// Shows what went wrong in place of every answer still streaming: the
/// agent itself could not run.
pub fn fail_streaming(error: &str) {
    state().update(|workspace| {
        for session in &mut workspace.sessions {
            for message in &mut session.messages {
                if message.streaming {
                    message.body = format!("Request failed: {error}").into();
                    message.streaming = false;
                }
            }
        }
    });
}

/// The messages of `session` whose thought shows open at first.
pub fn open_thoughts(session_id: u32) -> Vec<u32> {
    state().with(|workspace| {
        session(workspace, session_id)
            .map(|session| {
                session
                    .messages
                    .iter()
                    .filter(|message| message.reasoning_open)
                    .map(|message| message.id)
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
                    .flat_map(|message| message.tools.iter())
                    .filter(|tool| tool.open)
                    .map(|tool| tool.id.clone())
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
