//! What the store keeps between runs: the workspaces the app has open, the
//! sessions in them with their messages, and the ids the next ones take. It
//! sits in the app's storage beside the settings.

use nsh_core::{prelude::SharedString, storage::Storage};
use serde::{Deserialize, Serialize};

use crate::{Author, Message, Meta, Session, State, ToolChange, ToolRun, Workspace};

/// The storage key the state is kept under.
const KEY: &str = "aedt.store";

/// The state as it is kept.
#[derive(Deserialize, Serialize)]
struct Kept {
    workspaces: Vec<KeptWorkspace>,
    sessions: Vec<KeptSession>,
    selected: u32,
    next_session_id: u32,
    next_workspace_id: u32,
    next_message_id: u32,
}

#[derive(Deserialize, Serialize)]
struct KeptWorkspace {
    id: u32,
    path: String,
    name: String,
}

#[derive(Deserialize, Serialize)]
struct KeptSession {
    id: u32,
    title: String,
    workspace: Option<u32>,
    #[serde(default)]
    archived: bool,
    messages: Vec<KeptMessage>,
}

#[derive(Deserialize, Serialize)]
struct KeptMessage {
    id: u32,
    author: KeptAuthor,
    body: String,
    #[serde(default)]
    reasoning: Vec<String>,
    #[serde(default)]
    reasoning_open: bool,
    #[serde(default)]
    tools: Vec<KeptTool>,
    meta: Option<KeptMeta>,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum KeptAuthor {
    User,
    Assistant,
}

#[derive(Deserialize, Serialize)]
struct KeptTool {
    id: String,
    name: String,
    arguments: String,
    output: String,
    #[serde(default)]
    failed: bool,
    /// Whether its result was still to come when the state was written: kept
    /// to spot the calls a quit cut short.
    #[serde(default)]
    running: bool,
    #[serde(default)]
    open: bool,
    /// The files the call changed, for a call that edits.
    #[serde(default)]
    changes: Vec<KeptChange>,
}

/// One file a tool call changed, as it is kept.
#[derive(Deserialize, Serialize)]
struct KeptChange {
    path: String,
    added: usize,
    removed: usize,
    diff: String,
}

#[derive(Deserialize, Serialize)]
struct KeptMeta {
    time: String,
    detail: Option<String>,
}

/// The state the app kept, or none while there is none or it cannot be read.
pub(crate) fn load() -> Option<State> {
    let kept = Storage::get_item(KEY)?;
    match serde_json::from_str(&kept) {
        Ok(kept) => Some(into_state(kept)),
        Err(error) => {
            eprintln!("store: the kept state could not be read: {error}");
            None
        }
    }
}

/// Keeps `state` for the next run, or says why it could not.
pub(crate) fn save(state: &State) {
    let kept = Kept {
        workspaces: state.workspaces.iter().map(KeptWorkspace::from).collect(),
        sessions: state.sessions.iter().map(KeptSession::from).collect(),
        selected: state.selected,
        next_session_id: state.next_session_id,
        next_workspace_id: state.next_workspace_id,
        next_message_id: state.next_message_id,
    };
    let text = match serde_json::to_string(&kept) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("store: the state could not be written: {error}");
            return;
        }
    };
    if let Err(error) = Storage::set_item(KEY, text) {
        eprintln!("store: {error}");
    }
}

/// The state `kept` describes, ready to show: a reply the app quit during
/// keeps what it said, and the session on show is one that is still there.
fn into_state(kept: Kept) -> State {
    let sessions: Vec<Session> = kept
        .sessions
        .into_iter()
        .map(KeptSession::into_session)
        .collect();
    let selected = match sessions.iter().any(|session| session.id == kept.selected) {
        true => kept.selected,
        false => sessions.first().map_or(kept.selected, |session| session.id),
    };

    State {
        sessions,
        workspaces: kept
            .workspaces
            .into_iter()
            .map(KeptWorkspace::into_workspace)
            .collect(),
        selected,
        next_session_id: kept.next_session_id,
        next_workspace_id: kept.next_workspace_id,
        next_message_id: kept.next_message_id,
    }
}

impl KeptWorkspace {
    fn into_workspace(self) -> Workspace {
        Workspace {
            id: self.id,
            path: self.path.into(),
            name: self.name.into(),
        }
    }
}

impl KeptSession {
    fn into_session(self) -> Session {
        Session {
            id: self.id,
            title: self.title.into(),
            messages: self
                .messages
                .into_iter()
                .filter_map(KeptMessage::into_message)
                .collect(),
            workspace: self.workspace,
            archived: self.archived,
        }
    }
}

impl KeptMessage {
    /// The message, or none for a reply that never got a word: the app quit
    /// while its model was still starting it.
    fn into_message(self) -> Option<Message> {
        let body: SharedString = self.body.into();
        let reasoning: Vec<SharedString> = self.reasoning.into_iter().map(Into::into).collect();
        let tools: Vec<ToolRun> = self.tools.into_iter().map(KeptTool::into_tool).collect();
        if self.author == KeptAuthor::Assistant
            && body.is_empty()
            && reasoning.is_empty()
            && tools.is_empty()
        {
            return None;
        }

        Some(Message {
            id: self.id,
            author: self.author.into(),
            body,
            reasoning,
            reasoning_open: self.reasoning_open,
            tools,
            meta: self.meta.map(KeptMeta::into_meta),
            streaming: false,
        })
    }
}

impl KeptTool {
    /// The call, its result come or lost with a quit: one that was still
    /// running never got its answer, and reads as failed.
    fn into_tool(self) -> ToolRun {
        ToolRun {
            id: self.id.into(),
            name: self.name.into(),
            arguments: self.arguments.into(),
            output: self.output.into(),
            failed: self.failed || self.running,
            running: false,
            open: self.open,
            changes: self
                .changes
                .into_iter()
                .map(KeptChange::into_change)
                .collect(),
        }
    }
}

impl KeptChange {
    fn into_change(self) -> ToolChange {
        ToolChange {
            path: self.path.into(),
            added: self.added,
            removed: self.removed,
            diff: self.diff.into(),
        }
    }
}

impl KeptMeta {
    fn into_meta(self) -> Meta {
        Meta {
            time: self.time.into(),
            detail: self.detail.map(Into::into),
        }
    }
}

impl From<&Workspace> for KeptWorkspace {
    fn from(workspace: &Workspace) -> Self {
        Self {
            id: workspace.id,
            path: workspace.path.to_string(),
            name: workspace.name.to_string(),
        }
    }
}

impl From<&Session> for KeptSession {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id,
            title: session.title.to_string(),
            workspace: session.workspace,
            archived: session.archived,
            messages: session.messages.iter().map(KeptMessage::from).collect(),
        }
    }
}

impl From<&Message> for KeptMessage {
    fn from(message: &Message) -> Self {
        Self {
            id: message.id,
            author: message.author.into(),
            body: message.body.to_string(),
            reasoning: message
                .reasoning
                .iter()
                .map(|thought| thought.to_string())
                .collect(),
            reasoning_open: message.reasoning_open,
            tools: message.tools.iter().map(KeptTool::from).collect(),
            meta: message.meta.as_ref().map(KeptMeta::from),
        }
    }
}

impl From<&ToolRun> for KeptTool {
    fn from(tool: &ToolRun) -> Self {
        Self {
            id: tool.id.to_string(),
            name: tool.name.to_string(),
            arguments: tool.arguments.to_string(),
            output: tool.output.to_string(),
            failed: tool.failed,
            running: tool.running,
            open: tool.open,
            changes: tool.changes.iter().map(KeptChange::from).collect(),
        }
    }
}

impl From<&ToolChange> for KeptChange {
    fn from(change: &ToolChange) -> Self {
        Self {
            path: change.path.to_string(),
            added: change.added,
            removed: change.removed,
            diff: change.diff.to_string(),
        }
    }
}

impl From<&Meta> for KeptMeta {
    fn from(meta: &Meta) -> Self {
        Self {
            time: meta.time.to_string(),
            detail: meta.detail.as_ref().map(|detail| detail.to_string()),
        }
    }
}

impl From<Author> for KeptAuthor {
    fn from(author: Author) -> Self {
        match author {
            Author::User => Self::User,
            Author::Assistant => Self::Assistant,
        }
    }
}

impl From<KeptAuthor> for Author {
    fn from(author: KeptAuthor) -> Self {
        match author {
            KeptAuthor::User => Self::User,
            KeptAuthor::Assistant => Self::Assistant,
        }
    }
}
