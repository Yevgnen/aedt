//! The app's state: the sessions and their messages stay behind these
//! functions. A global is per surface, so each window has its own.

use nsh_core::prelude::*;

/// Who wrote a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Author {
    User,
    Assistant,
}

/// One message of a session.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Message {
    pub(crate) id: u32,
    pub(crate) author: Author,
    pub(crate) body: SharedString,
    pub(crate) meta: Option<SharedString>,
}

/// One conversation in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Session {
    pub(crate) id: u32,
    pub(crate) title: SharedString,
    pub(crate) messages: Vec<Message>,
}

impl Session {
    /// What the sidebar and the header show: an untitled session keeps the
    /// placeholder until its first message names it.
    pub(crate) fn display_title(&self) -> SharedString {
        if self.title.is_empty() {
            "新会话".into()
        } else {
            self.title.clone()
        }
    }
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
            title: "设计会话栏的布局".into(),
            messages: vec![
                message(
                    &mut next_message_id,
                    Author::User,
                    "先搭一个骨架，左边会话列表，右边聊天。",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "先定住三件事：\n\n1. 会话是唯一的领域对象，消息只属于会话；\n2. 列表用 `virtual_list`，行内不保存状态；\n3. 输入框的草稿属于页面，不放进 store。\n\n```rust\nlet draft = signal(String::new());\n```",
                    Some("Mock · 96 tokens · 0.3 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "输入的时候怎么回显 mock 数据？",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "草稿是页面的 signal：输入框绑定它，回显条也绑定它。每次按键只重跑这两处绑定，列表和侧栏都不动。",
                    Some("Mock · 64 tokens · 0.2 s"),
                ),
            ],
        };
        let second = Session {
            id: 2,
            title: "排查 nsh 窗口启动失败".into(),
            messages: vec![
                message(
                    &mut next_message_id,
                    Author::User,
                    "cargo run 起来后窗口是空的。",
                    None,
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "先确认根组件填满了窗口：给它 `.flex_1()` 和主题背景，否则窗口后面没有任何东西绘制。",
                    Some("Mock · 42 tokens · 0.2 s"),
                ),
            ],
        };
        let third = Session {
            id: 3,
            title: "给工具调用加超时".into(),
            messages: vec![message(
                &mut next_message_id,
                Author::Assistant,
                "每个工具调用都该有超时，并且把「超时」和「被取消」分开报给上层。",
                Some("Mock · 38 tokens · 0.1 s"),
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
    meta: Option<&'static str>,
) -> Message {
    let message = Message {
        id: *next_id,
        author,
        body: body.into(),
        meta: meta.map(SharedString::from),
    };
    *next_id += 1;
    message
}

fn workspace() -> Signal<Workspace> {
    global::<Workspace>()
}

/// How many sessions there are (tracked where it is read).
pub(crate) fn session_count() -> usize {
    workspace().with(|workspace| workspace.sessions.len())
}

/// The sessions' ids, in order (tracked where it is read).
pub(crate) fn session_ids() -> Vec<u32> {
    workspace().with(|workspace| {
        workspace
            .sessions
            .iter()
            .map(|session| session.id)
            .collect()
    })
}

/// The session at `index`, or none past the end (tracked where it is read).
pub(crate) fn session_at(index: usize) -> Option<Session> {
    workspace().with(|workspace| workspace.sessions.get(index).cloned())
}

/// One session, or none once it is gone (tracked where it is read).
pub(crate) fn session_by_id(id: u32) -> Option<Session> {
    workspace().with(|workspace| {
        workspace
            .sessions
            .iter()
            .find(|session| session.id == id)
            .cloned()
    })
}

/// The session the conversation pane shows (tracked where it is read).
pub(crate) fn selected_session_id() -> u32 {
    workspace().with(|workspace| workspace.selected)
}

/// Shows `id` in the conversation pane; an unknown id changes nothing.
pub(crate) fn select_session(id: u32) {
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
pub(crate) fn create_session() {
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
pub(crate) fn message_count(session_id: u32) -> usize {
    workspace().with(|workspace| {
        workspace
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .map_or(0, |session| session.messages.len())
    })
}

/// The messages' ids, in order (tracked where it is read).
pub(crate) fn message_ids(session_id: u32) -> Vec<u32> {
    workspace().with(|workspace| {
        workspace
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .map(|session| session.messages.iter().map(|message| message.id).collect())
            .unwrap_or_default()
    })
}

/// The message at `index`, or none past the end (tracked where it is read).
pub(crate) fn message_at(session_id: u32, index: usize) -> Option<Message> {
    workspace().with(|workspace| {
        workspace
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .and_then(|session| session.messages.get(index))
            .cloned()
    })
}

/// Sends `text` and appends the placeholder reply the mock model returns.
/// A blank message sends nothing.
pub(crate) fn send(session_id: u32, text: &str) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    let reply = mock_reply(text);
    workspace().update(|workspace| {
        let Some(index) = workspace
            .sessions
            .iter()
            .position(|session| session.id == session_id)
        else {
            return;
        };

        let user_id = workspace.next_message_id;
        let reply_id = workspace.next_message_id + 1;
        workspace.next_message_id += 2;

        let session = &mut workspace.sessions[index];
        if session.title.is_empty() {
            session.title = title_from(text);
        }
        session.messages.push(Message {
            id: user_id,
            author: Author::User,
            body: text.to_owned().into(),
            meta: None,
        });
        session.messages.push(Message {
            id: reply_id,
            author: Author::Assistant,
            body: reply.body,
            meta: Some(reply.meta),
        });
    });
}

struct MockReply {
    body: SharedString,
    meta: SharedString,
}

fn mock_reply(prompt: &str) -> MockReply {
    let snippet = preview(prompt, 24);
    let body = format!(
        "收到：{snippet}\n\n这是 **Mock 回复**，用来占位：\n\n```rust\nfn main() {{\n    println!(\"Aedt\");\n}}\n```\n\n接入真实模型后，把 `store::send` 换成流式请求即可。"
    );
    let tokens = prompt.chars().count() / 2 + 48;
    MockReply {
        body: body.into(),
        meta: format!("Mock · {tokens} tokens · 0.3 s").into(),
    }
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
