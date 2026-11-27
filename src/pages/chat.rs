//! The conversation: its header, its message timeline and its composer.

use std::collections::HashMap;

use aedt_settings::{self as settings, Thinking};
use aedt_store::{self as store, Author, Message, Part, ToolRun};
use aedt_ui::{MESSAGE_LINE, button, empty, icon, markdown, motion, sizes, tokens};
use nsh_base::theme;
use nsh_core::{LineHeight, element::desktop::title_bar, prelude::*};

mod composer;
mod rail;
mod tool;

use composer::composer;
use rail::rail;
use tool::tool_run;

/// One session's conversation. The caller keys it by the session, so
/// switching conversations resets the draft and the scroll position.
pub(crate) fn pane(session_id: u32, sidebar: Signal<bool>) -> View {
    let draft = signal(String::new());
    // What the reader folded open or away: a reply read with its thought open
    // starts in it, and a call shows its output where the store left it open.
    let folds = Folds::of(session_id);
    // The timeline's handle: the rail's ticks scroll it to their message.
    let timeline = element_ref();
    // Where the timeline stands, as it reports it: a wheel over the pane's blank
    // sides moves the list from there.
    let scrolled = signal(None::<f32>);
    // The reader's place: the last user message the list shows.
    let current = signal(store::last_user_index(session_id, 0, usize::MAX));
    let enter = motion().entering(motion::fade());

    view()
        .flex_1()
        .min_w(px(0.0))
        .bg(move || theme().background)
        .bind_motion(&enter)
        .child(header(session_id, sidebar))
        .child(
            view()
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .flex_col()
                // The messages and the rail beside them: the rail is centered on
                // this row, not on the composer under it.
                .child(
                    column(
                        // The blank sides of the messages' column, each taking the
                        // room the column leaves: a wheel over them scrolls the
                        // timeline, and the column itself keeps its width.
                        view()
                            .w_full()
                            .min_h(px(0.0))
                            .flex_row()
                            .child(blank_side(scrolled, &timeline))
                            .child(
                                view()
                                    .flex_none()
                                    .w_full()
                                    .max_w(px(sizes::COMPOSER_MAX))
                                    .min_h(px(0.0))
                                    .px(px(sizes::COMPOSER_SIDE))
                                    .child(timeline_view(
                                        session_id, folds, &timeline, current, scrolled,
                                    )),
                            )
                            .child(blank_side(scrolled, &timeline)),
                        rail(session_id, timeline, current),
                    )
                    .flex_1()
                    .min_h(px(0.0)),
                )
                .child(
                    column(
                        view()
                            .w_full()
                            .max_w(px(sizes::COMPOSER_MAX))
                            .px(px(sizes::COMPOSER_SIDE))
                            .child(composer(session_id, draft)),
                        view().w(px(sizes::RAIL)).flex_none(),
                    )
                    .flex_none(),
                ),
        )
}

/// One row of the conversation: its `content` centered in the pane, between a
/// gutter as wide as the rail and `aside` — the rail itself beside the
/// messages, an empty gutter beside the composer. The gutters keep the two
/// rows on one column whatever the pane's width.
fn column(content: impl IntoElement, aside: impl IntoElement) -> View {
    view()
        .w_full()
        .flex_row()
        .child(view().w(px(sizes::RAIL)).flex_none())
        .child(
            view()
                .flex_1()
                .min_w(px(0.0))
                .min_h(px(0.0))
                .flex_row()
                .justify_center()
                .child(content),
        )
        .child(aside)
}

/// What the reader folded open or away in a conversation: a thought of a
/// message, and a tool call.
#[derive(Clone, Copy)]
pub(super) struct Folds {
    /// The thoughts the reader opened, each by its message and its place in it.
    thoughts: Signal<Vec<(u32, usize)>>,
    /// What each call shows, by the call's id: absent until the reader decides.
    calls: Signal<HashMap<SharedString, bool>>,
}

impl Folds {
    /// The folds of `session`: a reply read with its thought open starts in it,
    /// and a call shows its output where the store left it open.
    fn of(session_id: u32) -> Self {
        Self {
            thoughts: signal(store::open_thoughts(session_id)),
            calls: signal(
                store::open_calls(session_id)
                    .into_iter()
                    .map(|call| (call, true))
                    .collect(),
            ),
        }
    }

    /// Whether the thought at `key` shows.
    fn thought_open(self, key: (u32, usize)) -> bool {
        self.thoughts.with(|thoughts| thoughts.contains(&key))
    }

    /// Opens or folds away the thought at `key`.
    fn toggle_thought(self, key: (u32, usize)) {
        self.thoughts.update(
            |thoughts| match thoughts.iter().position(|shown| *shown == key) {
                Some(at) => {
                    thoughts.remove(at);
                }
                None => thoughts.push(key),
            },
        );
    }

    /// Whether `tool` shows what it returned: the reader's decision, or, before
    /// they made one, whether it changed files.
    pub(super) fn call_open(self, tool: &ToolRun) -> bool {
        self.calls
            .with(|calls| calls.get(&tool.id).copied())
            .unwrap_or(!tool.changes.is_empty())
    }

    /// Shows or folds away the call `id`, as the reader decided.
    pub(super) fn set_call(self, id: SharedString, open: bool) {
        self.calls.update(|calls| {
            calls.insert(id, open);
        });
    }
}

fn header(session_id: u32, sidebar: Signal<bool>) -> impl IntoElement {
    view()
        .h(px(sizes::TOPBAR))
        .flex_row()
        .items_center()
        .pr(move || Length::from(title_bar().right + px(24.0)))
        .window_drag(true)
        .border_b_1()
        .border_b_color(move || theme().border)
        // While the sidebar is away, the pane takes its place: the toggle that
        // brings it back is right-aligned where the sidebar's own stands, so it
        // does not move as either appears.
        .child(when(
            move || !sidebar.get(),
            move || {
                view()
                    .w(px(sizes::SIDEBAR))
                    .flex_none()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .pr_3()
                    .child(
                        button::ghost_icon(icon::panel_left(14.0), "Show the sidebar")
                            .on_press(move || sidebar.set(true)),
                    )
            },
        ))
        .child(
            text(move || {
                store::session_summary(session_id)
                    .map(|session| session.title)
                    .unwrap_or_else(|| store::UNTITLED.into())
            })
            .flex_1()
            .min_w(px(0.0))
            .pl(px(24.0))
            .text_sm()
            .number_of_lines(1)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(move || theme().foreground),
        )
}

fn timeline_view(
    session_id: u32,
    folds: Folds,
    handle: &ElementRef,
    current: Signal<Option<usize>>,
    scrolled: Signal<Option<f32>>,
) -> impl IntoElement {
    virtual_list(
        move || store::message_count(session_id),
        move |at| message_row(session_id, folds, at),
    )
    .keys(move || store::message_ids(session_id))
    .element_ref(handle)
    .alignment(ListAlignment::Bottom)
    .scrollbar(false)
    .content_padding_x(px(sizes::CARD_PADDING))
    .content_padding_y(px(24.0))
    .estimated_item_size(px(row_height(session_id)))
    .row_gap(px(16.0))
    // Where it stands, for the wheel that scrolls it from the pane's sides.
    .on_scroll(move |event: &ScrollEvent| scrolled.set(Some(event.offset.y.0)))
    // The reader's place follows what the list shows as it scrolls.
    .on_viewable_items_change(move |items: &ViewableItems| {
        let place = items
            .viewable
            .and_then(|(first, last)| store::last_user_index(session_id, first, last));
        if current.get_untracked() != place {
            current.set(place);
        }
    })
    .empty(empty::empty("Say something…"))
    .flex_1()
}

/// The points one line of a mouse wheel scrolls the timeline, when the wheel is
/// over the pane's blank sides: about one line of a message's prose.
const WHEEL_LINE: f32 = sizes::TEXT_COMPACT * MESSAGE_LINE;

/// A blank side of the messages' column: a wheel over it scrolls the timeline as
/// one over the timeline itself does, from where the list last said it stands.
fn blank_side(scrolled: Signal<Option<f32>>, timeline: &ElementRef) -> impl IntoElement {
    let timeline = *timeline;

    view()
        .flex_1()
        .min_w(px(0.0))
        .on_wheel(move |event: &WheelEvent| {
            // Nothing to move from until the list has reported where it is: it
            // does so as it lays out, at the end it opens on.
            let Some(at) = scrolled.get_untracked() else {
                return;
            };
            let delta = event.delta.points(px(WHEEL_LINE)).y.0;
            // Recorded before the list reports it, so wheels that come before
            // the report move on from where they left it rather than from
            // where the list still is.
            let next = (at + delta).max(0.0);
            scrolled.set(Some(next));

            timeline.scroll_to_offset(px(next), false);
        })
}

/// How tall a row is guessed at before the list measures it, by what the row
/// holds: a message the reader wrote, a thought's bar or a call's row, a line
/// of a reply's prose, and the head and lines of a changed file.
const ROW_USER: f32 = 76.0;
const ROW_PART: f32 = 26.0;
const ROW_PROSE_LINE: f32 = 20.0;
const ROW_DIFF_HEAD: f32 = 30.0;
const ROW_DIFF_LINE: f32 = 17.0;

/// The characters a line of a reply's prose holds, on average.
const PROSE_PER_LINE: usize = 80;

/// A row's height as the list guesses it before measuring it: how tall a
/// message of this conversation tends to be. The list maps a scroll offset to
/// rows by these sizes, so a guess far below the rows makes it bind a whole
/// conversation to show one screen of it, and one far above leaves the screen
/// empty while it catches up. Read once, as the conversation opens: a message
/// that arrives later only moves the average a little.
fn row_height(session_id: u32) -> f32 {
    let messages: Vec<Message> = untracked(|| {
        (0..store::message_count(session_id))
            .filter_map(|at| store::message_at(session_id, at))
            .collect()
    });
    let tall: f32 = messages.iter().map(message_height).sum();

    tall / messages.len().max(1) as f32
}

/// How tall a message's row is before it is measured, by what it holds.
fn message_height(message: &Message) -> f32 {
    if message.author == Author::User {
        return ROW_USER;
    }

    message.parts.iter().fold(0.0, |tall, part| match part {
        Part::Thought(_) => tall + ROW_PART,
        Part::Text(text) => {
            tall + ROW_PROSE_LINE * (text.chars().count() / PROSE_PER_LINE + 1) as f32
        }
        // The files a call changed show open, so their lines count; a call
        // that changed none shows its row alone.
        Part::Tool(tool) => {
            let lines: usize = tool
                .changes
                .iter()
                .map(|change| change.diff.lines().count())
                .sum();

            tall + ROW_PART
                + ROW_DIFF_HEAD * tool.changes.len() as f32
                + ROW_DIFF_LINE * lines as f32
        }
    })
}

/// The column of question ticks at the pane's right edge: one per user message,
/// all light and short at rest. The lit ones — under the pointer, and the last
/// question the list shows — are dark and long; a press jumps to it.
/// What `body` says, as the preview's text binds it.
pub(super) fn body_of(body: Memo<Option<Message>>) -> impl Fn() -> SharedString + Copy + 'static {
    move || {
        body.with(|message| {
            message
                .as_ref()
                .map(|message| SharedString::from(message.body()))
                .unwrap_or_default()
        })
    }
}

/// A cell, made once and given other messages as it is reused.
fn message_row(session_id: u32, folds: Folds, at: Position) -> impl IntoElement {
    let message = computed(move || store::message_at(session_id, at.index()));
    let from_user =
        move || message.with(|message| message.as_ref().is_some_and(|m| m.author == Author::User));

    view().w_full().child(
        when(from_user, move || user_message(message))
            .otherwise(move || assistant_message(message, folds)),
    )
}

fn user_message(message: Memo<Option<Message>>) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        .child(
            view().w_full().flex_row().justify_end().child(
                view()
                    .max_w(px(sizes::MESSAGE_MAX))
                    .rounded(move || theme().radius)
                    .bg(tokens::bubble)
                    .px_4()
                    .py_3()
                    .child(
                        text(move || {
                            message.with(|message| {
                                message
                                    .as_ref()
                                    .map(|message| SharedString::from(message.body()))
                                    .unwrap_or_default()
                            })
                        })
                        .text_sm()
                        .line_height(LineHeight::Multiple(MESSAGE_LINE))
                        .selectable(true)
                        .text_color(move || theme().primary_foreground),
                    ),
            ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_end()
                .gap_1()
                .child(message_meta(message))
                .child(copy_message(message)),
        )
}

/// A message's trailing action: puts its text on the clipboard.
fn copy_message(message: Memo<Option<Message>>) -> impl IntoElement {
    let copy = move || {
        let body = message.with(|message| {
            message
                .as_ref()
                .map(|message| message.body())
                .unwrap_or_default()
        });
        if body.is_empty() {
            return;
        }
        let write = set_clipboard(body);
        spawn(async move {
            if write.await == ClipboardWrite::Failed {
                eprintln!("copy message: the clipboard refused the write");
            }
        });
    };

    button::ghost_icon(icon::copy(14.0), "Copy message")
        .text_color(move || tokens::caption().opacity(0.8))
        .on_press(copy)
}

/// The muted line under a message: when it was sent, and for a reply the model
/// it ran on and how long it took. The caller aligns it; it reads the message
/// where that message now is.
fn message_meta(message: Memo<Option<Message>>) -> impl IntoElement {
    when(
        move || {
            message.with(|message| {
                message
                    .as_ref()
                    .is_some_and(|message| message.meta.is_some())
            })
        },
        move || {
            text(move || {
                message.with(|message| {
                    message
                        .as_ref()
                        .and_then(|message| message.meta.as_ref())
                        .map(|meta| meta.time.clone())
                        .unwrap_or_default()
                })
            })
            .text_xs()
            .text_color(move || tokens::caption().opacity(0.8))
        },
    )
}

/// What a reply adds to that line: the model it ran on and how long it took.
fn message_detail(message: Memo<Option<Message>>) -> impl IntoElement {
    text(move || {
        message.with(|message| {
            message
                .as_ref()
                .and_then(|message| message.meta.as_ref())
                .and_then(|meta| meta.detail.clone())
                .map(|detail| format!("· {detail}"))
                .unwrap_or_default()
        })
    })
    .text_xs()
    .text_color(move || tokens::caption().opacity(0.8))
}

fn assistant_message(message: Memo<Option<Message>>, folds: Folds) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        // The reply in the order it happened: what it thought, the call it
        // made, what it answered, and so on, each where it came.
        .child(each(
            // The reply's parts by their place and what each is: a row reads
            // the part it shows from the message, so the list does not copy
            // every part of every reply each time it is built.
            move || {
                message.with(|message| {
                    message.as_ref().map_or_else(Vec::new, |message| {
                        message
                            .parts
                            .iter()
                            .enumerate()
                            .map(|(at, part)| (at, PartKind::of(part)))
                            .collect()
                    })
                })
            },
            |part| *part,
            move |part, _| {
                let (at, kind) = part.with_untracked(|part| *part);
                part_of(message, at, kind, folds)
            },
        ))
        .child(
            // The model and the duration show while the pointer is over the row
            // they share; when the reply was sent always shows.
            view()
                .group()
                .flex_row()
                .items_center()
                .gap_1()
                .child(copy_message(message))
                .child(message_meta(message))
                .child(
                    view()
                        .opacity(0.0)
                        .group_hover(|s| s.opacity(1.0))
                        .group_focus_within(|s| s.opacity(1.0))
                        .transition_opacity()
                        .duration(motion::row_duration())
                        .ease_out()
                        .child(message_detail(message)),
                ),
        )
}

/// What a part of a reply is, which decides how its row is drawn.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PartKind {
    Thought,
    Text,
    Tool,
}

impl PartKind {
    fn of(part: &Part) -> Self {
        match part {
            Part::Thought(_) => Self::Thought,
            Part::Text(_) => Self::Text,
            Part::Tool(_) => Self::Tool,
        }
    }
}

/// One stretch of the reply, drawn by what it is: a thought, a call, or a
/// stretch of what it answered. The kind is the row's identity along with its
/// place, and not read here: the list gives a cell another message as it
/// scrolls, and a row keyed only by its place would keep the kind of the
/// message before it.
fn part_of(
    message: Memo<Option<Message>>,
    at: usize,
    kind: PartKind,
    folds: Folds,
) -> impl IntoElement {
    match kind {
        PartKind::Thought => thought(message, at, folds).into_element(),
        PartKind::Text => text_part(message, at).into_element(),
        PartKind::Tool => {
            // The call this part holds, which is the one the card reads as it
            // runs and once its result is in.
            let tool = computed(move || {
                message.with(|message| {
                    message
                        .as_ref()
                        .and_then(|message| match message.parts.get(at) {
                            Some(Part::Tool(tool)) => Some(tool.clone()),
                            _ => None,
                        })
                        .unwrap_or_default()
                })
            });

            tool_run(tool, folds).into_element()
        }
    }
}

/// A stretch of the reply's answer, read as Markdown, the last one while it
/// still arrives as one being written.
fn text_part(message: Memo<Option<Message>>, at: usize) -> impl IntoElement {
    markdown::body(
        move || {
            message.with(|message| {
                message
                    .as_ref()
                    .and_then(|message| match message.parts.get(at) {
                        Some(Part::Text(text)) => Some(text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default()
            })
        },
        // While the stretch is still arriving, the text view shows its open
        // syntax as closed and again only the blocks that changed.
        move || {
            message.with(|message| {
                message
                    .as_ref()
                    .is_some_and(|message| message.streaming && message.parts.len() == at + 1)
            })
        },
    )
}

/// What the thought at `at` says, trimmed, while that part is a thought.
fn thought_text(message: Memo<Option<Message>>, at: usize) -> Option<String> {
    message.with(|message| {
        message
            .as_ref()
            .and_then(|message| match message.parts.get(at) {
                Some(Part::Thought(text)) => Some(text.trim_end().to_owned()),
                _ => None,
            })
    })
}

/// Whether the thought at `at` has arrived.
fn has_thought(message: Memo<Option<Message>>, at: usize) -> bool {
    thought_text(message, at).is_some_and(|text| !text.is_empty())
}

/// Whether the thought at `at` is the one arriving: it is the last part of a
/// reply still being written.
fn writing(message: Memo<Option<Message>>, at: usize) -> bool {
    message.with(|message| {
        message
            .as_ref()
            .is_some_and(|message| message.streaming && message.parts.len() == at + 1)
    })
}

/// One thought of the reply: its text under the quiet bar that folds it away,
/// shown as the reader's settings ask — folded, open, open while it arrives,
/// or its last lines. A part nothing reasoned in shows nothing.
fn thought(message: Memo<Option<Message>>, at: usize, folds: Folds) -> impl IntoElement {
    let key = move || message.with(|message| message.as_ref().map(|message| (message.id, at)));
    let shown = move || {
        let opened = key().is_some_and(|key| folds.thought_open(key));

        match settings::thinking() {
            // Folded away, even while it arrives.
            Thinking::Collapsed => opened,
            // Always open: the reader's fold is not what decides here.
            Thinking::Expanded | Thinking::Preview => true,
            // Open while it arrives, and after while the reader leaves it open.
            Thinking::Auto => opened || writing(message, at),
        }
    };

    when(
        move || has_thought(message, at),
        move || {
            view()
                .w_full()
                .gap_1()
                .child(thinking_bar(key, folds, shown))
                .child(when(shown, move || thinking_body(message, at)))
        },
    )
}

/// The quiet row over a thought: its bulb and its name, and the chevron that
/// folds it away while the mode leaves that to the reader.
fn thinking_bar(
    key: impl Fn() -> Option<(u32, usize)> + Copy + 'static,
    folds: Folds,
    shown: impl Fn() -> bool + Copy + 'static,
) -> impl IntoElement {
    let toggle = move || {
        let Some(key) = key() else {
            return;
        };
        folds.toggle_thought(key);
    };

    view()
        .w_full()
        .group()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_2()
        .text_color(tokens::meta)
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .px_1()
                .py(px(2.0))
                .child(icon::bulb(14.0))
                .child(text("Thinking").text_xs()),
        )
        .child(when(
            // A mode that always shows the thought leaves nothing to fold.
            move || !matches!(settings::thinking(), Thinking::Expanded | Thinking::Preview),
            move || {
                button::ghost_icon(
                    when(shown, || icon::chevron_up(14.0)).otherwise(|| icon::chevron_down(14.0)),
                    "Expand or collapse the thinking",
                )
                // The chevron shows while the pointer is over the thought's row,
                // and for the keyboard's focus, as a call's does.
                .opacity(0.0)
                .group_hover(|s| s.opacity(1.0))
                .group_focus_within(|s| s.opacity(1.0))
                .on_press(toggle)
            },
        ))
}

/// What a thought shows: its last lines in the preview mode, all of it
/// otherwise.
fn thinking_body(message: Memo<Option<Message>>, at: usize) -> impl IntoElement {
    when(
        move || settings::thinking() == Thinking::Preview,
        move || thinking_preview(message, at).into_element(),
    )
    .otherwise(move || thinking_text(message, at).into_element())
}

/// Where a thought's text starts: under the word beside its bulb, past the
/// row's inset, the mark and the gap between them.
const THINKING_TEXT_INSET: f32 = 26.0;

/// The thought's own text size, and the blank line kept under it: one line of
/// that text, so the answer opens a line below the thought.
const THINKING_LINE: f32 = sizes::TEXT_COMPACT * MESSAGE_LINE;

/// The lines of a thought the preview keeps in view.
const PREVIEW_LINES: f32 = 6.0;

/// The thought's own text, quieter than the answer it led to.
fn thinking_text(message: Memo<Option<Message>>, at: usize) -> impl IntoElement {
    text(move || thought_text(message, at).unwrap_or_default())
        .w_full()
        .pl(px(THINKING_TEXT_INSET))
        .pb(px(THINKING_LINE))
        .text_size(px(sizes::TEXT_COMPACT))
        .line_height(LineHeight::Multiple(MESSAGE_LINE))
        .text_color(move || tokens::meta().opacity(0.7))
}

/// The thought's last lines, up to [`PREVIEW_LINES`] of them: the box grows
/// with the text until it holds that many, and from then on the end of what
/// arrived stays at its bottom while what came before is pushed up out of it.
fn thinking_preview(message: Memo<Option<Message>>, at: usize) -> impl IntoElement {
    view()
        .w_full()
        .pl(px(THINKING_TEXT_INSET))
        .pb(px(THINKING_LINE))
        .child(
            view()
                .w_full()
                .max_h(px(PREVIEW_LINES * THINKING_LINE))
                .flex_col()
                // The text keeps its own height past the box's, so its end sits
                // at the box's bottom and its start is what the box clips.
                .justify_end()
                .overflow_hidden()
                .child(
                    text(move || thought_text(message, at).unwrap_or_default())
                        .w_full()
                        .flex_none()
                        .text_size(px(sizes::TEXT_COMPACT))
                        .line_height(LineHeight::Multiple(MESSAGE_LINE))
                        .text_color(move || tokens::meta().opacity(0.7)),
                ),
        )
}
