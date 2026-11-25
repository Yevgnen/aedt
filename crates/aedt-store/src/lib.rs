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
    /// The chain of thought the reply reasoned with, shown above it: empty for
    /// a message that did not think.
    pub reasoning: SharedString,
    /// The muted line under the message: when it was sent, and for a reply what
    /// it ran on and how long it took. `None` shows no line at all.
    pub meta: Option<Meta>,
    /// Whether the text is still arriving: a reply's text view then reads it as
    /// one being written, so only the blocks that changed are shown again
    /// instead of the whole document on every token.
    pub streaming: bool,
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
struct Workspace {
    sessions: Vec<Session>,
    selected: u32,
    next_session_id: u32,
    next_message_id: u32,
}

impl Default for Workspace {
    fn default() -> Self {
        let mut next_message_id = 1;
        let first = Session {
            id: 1,
            title: "Laying out the session sidebar".into(),
            messages: vec![
                message(
                    &mut next_message_id,
                    Author::User,
                    "Start with a skeleton: the session list on the left, the chat on the right.",
                    "11/25 10:12",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "Pin three things down:\n\n1. The session is the only domain object, and messages belong to it;\n2. The list uses `virtual_list`, and no row keeps state;\n3. The composer's draft belongs to the page, not to the store.\n\n```rust\nlet draft = signal(String::new());\n```",
                    "11/25 10:12",
                    Some("deepseek-flash · 0.3 s"),
                ),
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
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "Because `virtual_list` reuses its cells: one cell shows different rows in turn. A row's signal travels with the cell to the next row, and states bleed. Keep each row's state in the data, keyed by message id; the cell reads only the one at its position.",
                    "11/25 10:17",
                    Some("deepseek-flash · 0.3 s"),
                ),
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
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "Each row compares its session id with the selected one: the selected row takes the `accent` fill and `foreground` text, and a lighter wash on hover. No left bar, and no bold text standing in for selection.",
                    "11/25 10:24",
                    Some("deepseek-flash · 0.2 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "Can a code block in a long reply burst the message column?",
                    "11/25 10:28",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "An assistant message renders Markdown with `text_view`, and a code block scrolls sideways in its own area, so it never bursts the column; a user message stays plain text, at most 560pt wide.\n\n```rust\nfn main() {\n    println!(\"Aedt\");\n}\n```\n\nThis long body is here to push the list into scrolling.",
                    "11/25 10:28",
                    Some("deepseek-flash · 0.4 s"),
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
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "That the keyboard can finish the main flow, focus is always visible, the empty, loading and failed states all exist, both light and dark have been seen, and the smallest window does not break the layout. Then run a real window on the target platform, not just screenshots.",
                    "11/25 10:37",
                    Some("deepseek-flash · 0.2 s"),
                ),
            ],
        };
        let second = Session {
            id: 2,
            title: "Debugging an nsh window that won't start".into(),
            messages: vec![
                message(
                    &mut next_message_id,
                    Author::User,
                    "The window is empty after `cargo run`.",
                    "11/25 09:41",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "First check that the root fills the window: give it `.flex_1()` and the theme's background, or nothing gets drawn behind the window.",
                    "11/25 09:41",
                    Some("deepseek-flash · 0.2 s"),
                ),
            ],
        };
        let third = Session {
            id: 3,
            title: "Timeouts for tool calls".into(),
            messages: vec![message(
                &mut next_message_id,
                Author::Assistant,
                "Every tool call needs a timeout, and a timeout must reach the layer above apart from a cancellation.",
                "11/25 08:15",
                Some("deepseek-flash · 0.1 s"),
            )],
        };

        Self {
            sessions: vec![first, second, third],
            selected: 1,
            next_session_id: 4,
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
        reasoning: SharedString::default(),
        meta: Some(Meta {
            time: time.into(),
            detail: detail.map(SharedString::from),
        }),
        streaming: false,
    };
    *next_id += 1;
    message
}

/// The clock time a message shows, in the machine's own time zone: `11/6
/// 21:12` (month and day without a leading zero).
fn now_label() -> SharedString {
    Local::now().format("%-m/%-d %H:%M").to_string().into()
}

fn workspace() -> Signal<Workspace> {
    global::<Workspace>()
}

/// Session `id` of `workspace`.
fn session(workspace: &Workspace, id: u32) -> Option<&Session> {
    workspace.sessions.iter().find(|session| session.id == id)
}

/// Session `id` of `workspace`, to change.
fn session_mut(workspace: &mut Workspace, id: u32) -> Option<&mut Session> {
    workspace
        .sessions
        .iter_mut()
        .find(|session| session.id == id)
}

/// How many sessions there are (tracked where it is read).
pub fn session_count() -> usize {
    workspace().with(|workspace| workspace.sessions.len())
}

/// The sessions' ids, in order (tracked where it is read).
pub fn session_ids() -> Vec<u32> {
    workspace().with(|workspace| {
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
    workspace().with(|workspace| workspace.sessions.get(index).map(Session::summary))
}

/// What the session `id` shows, or none once it is gone (tracked where it is
/// read).
pub fn session_summary(id: u32) -> Option<SessionSummary> {
    workspace().with(|workspace| session(workspace, id).map(Session::summary))
}

/// The session the conversation pane shows (tracked where it is read).
pub fn selected_session_id() -> u32 {
    workspace().with(|workspace| workspace.selected)
}

/// Shows `id` in the conversation pane; an unknown id changes nothing.
pub fn select_session(id: u32) {
    let (current, exists) = workspace().with(|workspace| {
        (
            workspace.selected,
            workspace.sessions.iter().any(|session| session.id == id),
        )
    });
    if current == id || !exists {
        return;
    }
    workspace().update(|workspace| workspace.selected = id);
}

/// Adds an empty session above the others and shows it.
pub fn create_session() {
    let id = workspace().with(|workspace| workspace.next_session_id);
    workspace().update(|workspace| {
        workspace.next_session_id += 1;
        workspace.sessions.insert(
            0,
            Session {
                id,
                title: "".into(),
                messages: Vec::new(),
            },
        );
        workspace.selected = id;
    });
}

/// How many messages a session has (tracked where it is read).
pub fn message_count(session_id: u32) -> usize {
    workspace().with(|workspace| {
        session(workspace, session_id).map_or(0, |session| session.messages.len())
    })
}

/// The messages' ids, in order (tracked where it is read).
pub fn message_ids(session_id: u32) -> Vec<u32> {
    workspace().with(|workspace| {
        session(workspace, session_id)
            .map(|session| session.messages.iter().map(|message| message.id).collect())
            .unwrap_or_default()
    })
}

/// The message at `index`, or none past the end (tracked where it is read).
pub fn message_at(session_id: u32, index: usize) -> Option<Message> {
    workspace().with(|workspace| {
        session(workspace, session_id)
            .and_then(|session| session.messages.get(index))
            .cloned()
    })
}

/// The last user message among the rows `first..=last`: the one the rail marks
/// as the reader's current place.
pub fn last_user_index(session_id: u32, first: usize, last: usize) -> Option<usize> {
    workspace().with(|workspace| {
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
    workspace().with(|workspace| {
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
    workspace().with(|workspace| {
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
    workspace().with(|workspace| {
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
    workspace().update(|workspace| started = append_sent(workspace, session_id, text));

    started.map(|(reply_id, history)| Sent { reply_id, history })
}

/// The user's message, the reply left empty, and what the model gets to read:
/// every message before that reply.
fn append_sent(
    workspace: &mut Workspace,
    session_id: u32,
    text: &str,
) -> Option<(u32, Vec<Message>)> {
    let user_id = workspace.next_message_id;
    let reply_id = user_id + 1;
    workspace.next_message_id += 2;

    let sent_at = now_label();
    let session = session_mut(workspace, session_id)?;
    if session.title.is_empty() {
        session.title = title_from(text);
    }
    session.messages.push(Message {
        id: user_id,
        author: Author::User,
        body: text.to_owned().into(),
        reasoning: SharedString::default(),
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
        reasoning: SharedString::default(),
        meta: Some(Meta {
            time: sent_at,
            detail: None,
        }),
        streaming: true,
    });

    Some((reply_id, history))
}

/// Appends what arrived of the reply to `message_id`: its thinking first, then
/// its answer.
pub fn append_reply(session_id: u32, message_id: u32, thinking: &str, answer: &str) {
    if thinking.is_empty() && answer.is_empty() {
        return;
    }
    workspace().update(|workspace| {
        let Some(message) = find_message(workspace, session_id, message_id) else {
            return;
        };
        if !thinking.is_empty() {
            let mut reasoned = message.reasoning.to_string();
            reasoned.push_str(thinking);
            message.reasoning = reasoned.into();
        }
        if !answer.is_empty() {
            let mut body = message.body.to_string();
            body.push_str(answer);
            message.body = body.into();
        }
    });
}

/// Closes the reply: its meta line gains the model and how long the answer
/// took, after the time it was sent.
pub fn finish_reply(session_id: u32, message_id: u32, model: &str, seconds: f32) {
    workspace().update(|workspace| {
        let Some(message) = find_message(workspace, session_id, message_id) else {
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

/// Shows what went wrong in place of the answer.
pub fn fail_reply(session_id: u32, message_id: u32, error: &str) {
    workspace().update(|workspace| {
        let Some(message) = find_message(workspace, session_id, message_id) else {
            return;
        };
        message.body = format!("Request failed: {error}").into();
        message.streaming = false;
    });
}

/// Message `message_id` of session `session_id`.
fn find_message(
    workspace: &mut Workspace,
    session_id: u32,
    message_id: u32,
) -> Option<&mut Message> {
    session_mut(workspace, session_id)?
        .messages
        .iter_mut()
        .find(|message| message.id == message_id)
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
