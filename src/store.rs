//! The app's state: the sessions and their messages stay behind these
//! functions. A global is per surface, so each window has its own.

use chrono::Local;
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
    /// The chain of thought the reply reasoned with, shown above it: empty for
    /// a message that did not think.
    pub(crate) reasoning: SharedString,
    /// The muted line under the message: when it was sent, and for a reply what
    /// it ran on and how long it took. `None` shows no line at all.
    pub(crate) meta: Option<SharedString>,
    /// Whether the text is still arriving: a reply's text view then reads it as
    /// one being written, so only the blocks that changed are shown again
    /// instead of the whole document on every token.
    pub(crate) streaming: bool,
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
pub(crate) struct SessionSummary {
    pub(crate) id: u32,
    pub(crate) title: SharedString,
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
                    Some("11/25 10:12"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "先定住三件事：\n\n1. 会话是唯一的领域对象，消息只属于会话；\n2. 列表用 `virtual_list`，行内不保存状态；\n3. 输入框的草稿属于页面，不放进 store。\n\n```rust\nlet draft = signal(String::new());\n```",
                    Some("11/25 10:12 · deepseek-flash · 0.3 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "输入的时候怎么回显 mock 数据？",
                    Some("11/25 10:14"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "草稿是页面的 signal：输入框绑定它，回显条也绑定它。每次按键只重跑这两处绑定，列表和侧栏都不动。",
                    Some("11/25 10:14 · deepseek-flash · 0.2 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "消息列表滚动时，为什么不能把每行的状态放进行内的 signal？",
                    Some("11/25 10:17"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "因为 `virtual_list` 的 cell 会复用：同一个 cell 依次显示不同的行。行内 signal 会跟着 cell 走到下一行，状态就串了。每行的状态按消息 id 放在数据里，cell 只读当前位置的那一条。",
                    Some("11/25 10:17 · deepseek-flash · 0.3 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "会话切换后草稿为什么没了？",
                    Some("11/25 10:21"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "会话面板按 id 用 `dynamic` 重建，草稿 signal 和滚动位置都属于那一次构建，切换时一起释放。要保留的话，把草稿按会话 id 存进 store，面板打开时再恢复。",
                    Some("11/25 10:21 · deepseek-flash · 0.2 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "侧栏选中态你是怎么画的？",
                    Some("11/25 10:24"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "每行拿自己的会话 id 和当前选中 id 比较：选中时用 `accent` 底色和 `foreground` 文字，悬停是更浅的一层 wash。没有左侧竖条，也不靠加粗单独表达选中。",
                    Some("11/25 10:24 · deepseek-flash · 0.2 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "长回复里的代码块会不会把消息列撑破？",
                    Some("11/25 10:28"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "助手消息用 `text_view` 渲染 Markdown，代码块在自己的横向滚动区里，不会撑破消息列；用户消息保持纯文本，最宽 560pt。\n\n```rust\nfn main() {\n    println!(\"Aedt\");\n}\n```\n\n这段长内容就是用来把列表撑出滚动的。",
                    Some("11/25 10:28 · deepseek-flash · 0.4 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "深色模式还要额外改颜色吗？",
                    Some("11/25 10:33"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "不用。颜色都从 `theme()` 读，`card`、`border`、`muted` 各有明暗两套值；页面里不写 hex，缺角色就先补到主题里。",
                    Some("11/25 10:33 · deepseek-flash · 0.2 s"),
                ),
                message(
                    &mut next_message_id,
                    Author::User,
                    "发布前还要检查什么？",
                    Some("11/25 10:37"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "键盘能走完主流程、焦点始终可见、空/加载/失败态都有、浅色和深色都看过、窗口缩到最小不破版。最后在目标平台跑一次真实窗口，而不是只看截图。",
                    Some("11/25 10:37 · deepseek-flash · 0.2 s"),
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
                    Some("11/25 09:41"),
                ),
                message(
                    &mut next_message_id,
                    Author::Assistant,
                    "先确认根组件填满了窗口：给它 `.flex_1()` 和主题背景，否则窗口后面没有任何东西绘制。",
                    Some("11/25 09:41 · deepseek-flash · 0.2 s"),
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
                Some("11/25 08:15 · deepseek-flash · 0.1 s"),
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
        reasoning: SharedString::default(),
        meta: meta.map(SharedString::from),
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

/// What the session at `index` shows, or none past the end (tracked where it is
/// read).
pub(crate) fn session_summary_at(index: usize) -> Option<SessionSummary> {
    workspace().with(|workspace| workspace.sessions.get(index).map(Session::summary))
}

/// What the session `id` shows, or none once it is gone (tracked where it is
/// read).
pub(crate) fn session_summary(id: u32) -> Option<SessionSummary> {
    workspace().with(|workspace| {
        workspace
            .sessions
            .iter()
            .find(|session| session.id == id)
            .map(Session::summary)
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

/// The last user message among the rows `first..=last`: the one the rail marks
/// as the reader's current place.
pub(crate) fn last_user_index(session_id: u32, first: usize, last: usize) -> Option<usize> {
    workspace().with(|workspace| {
        let session = workspace
            .sessions
            .iter()
            .find(|session| session.id == session_id)?;
        let last = last.min(session.messages.len().checked_sub(1)?);
        (first..=last)
            .rev()
            .find(|index| session.messages[*index].author == Author::User)
    })
}

/// How many user messages the session has: one rail tick each (tracked where it
/// is read).
pub(crate) fn user_message_count(session_id: u32) -> usize {
    workspace().with(|workspace| {
        workspace
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .map_or(0, |session| {
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
pub(crate) fn user_message_indices(session_id: u32) -> Vec<usize> {
    workspace().with(|workspace| {
        workspace
            .sessions
            .iter()
            .find(|session| session.id == session_id)
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
pub(crate) fn reply_at(session_id: u32, index: usize) -> Option<Message> {
    workspace().with(|workspace| {
        let session = workspace
            .sessions
            .iter()
            .find(|session| session.id == session_id)?;
        session
            .messages
            .iter()
            .skip(index + 1)
            .take_while(|message| message.author == Author::Assistant)
            .next()
            .cloned()
    })
}

/// Sends `text`: appends the user's message and an empty reply, then streams
/// the model's answer into that reply. A blank message sends nothing.
pub(crate) fn send(session_id: u32, text: &str) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    let mut started = None;
    workspace().update(|workspace| started = append_sent(workspace, session_id, text));

    if let Some((reply_id, history)) = started {
        let model = crate::settings::selected();
        crate::llm::reply(session_id, reply_id, history, &model);
    }
}

/// The user's message, the reply left empty, and what the model gets to read:
/// every message before that reply.
fn append_sent(
    workspace: &mut Workspace,
    session_id: u32,
    text: &str,
) -> Option<(u32, Vec<Message>)> {
    let index = workspace
        .sessions
        .iter()
        .position(|session| session.id == session_id)?;

    let user_id = workspace.next_message_id;
    let reply_id = user_id + 1;
    workspace.next_message_id += 2;

    let sent_at = now_label();
    let session = &mut workspace.sessions[index];
    if session.title.is_empty() {
        session.title = title_from(text);
    }
    session.messages.push(Message {
        id: user_id,
        author: Author::User,
        body: text.to_owned().into(),
        reasoning: SharedString::default(),
        meta: Some(sent_at.clone()),
        streaming: false,
    });
    let history = session.messages.clone();
    session.messages.push(Message {
        id: reply_id,
        author: Author::Assistant,
        body: SharedString::default(),
        reasoning: SharedString::default(),
        meta: Some(sent_at),
        streaming: true,
    });

    Some((reply_id, history))
}

/// Appends what arrived of the reply to `message_id`: its thinking first, then
/// its answer.
pub(crate) fn append_reply(session_id: u32, message_id: u32, thinking: &str, answer: &str) {
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
pub(crate) fn finish_reply(session_id: u32, message_id: u32, model: &str, seconds: f32) {
    workspace().update(|workspace| {
        let Some(message) = find_message(workspace, session_id, message_id) else {
            return;
        };
        let sent_at = message.meta.take().unwrap_or_default();
        message.streaming = false;
        message.meta = Some(if sent_at.is_empty() {
            format!("{model} · {seconds:.1} s").into()
        } else {
            format!("{sent_at} · {model} · {seconds:.1} s").into()
        });
    });
}

/// Shows what went wrong in place of the answer.
pub(crate) fn fail_reply(session_id: u32, message_id: u32, error: &str) {
    workspace().update(|workspace| {
        let Some(message) = find_message(workspace, session_id, message_id) else {
            return;
        };
        message.body = format!("请求失败：{error}").into();
        message.streaming = false;
    });
}

/// Message `message_id` of session `session_id`.
fn find_message(
    workspace: &mut Workspace,
    session_id: u32,
    message_id: u32,
) -> Option<&mut Message> {
    workspace
        .sessions
        .iter_mut()
        .find(|session| session.id == session_id)?
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
