//! The conversation: its header, its message timeline and its composer.

use std::time::Duration;

use aedt_provider::responses::Effort;
use aedt_settings as settings;
use aedt_store::{self as store, Author, Message, Part, ToolRun};
use aedt_ui::{MESSAGE_LINE, badge, button, empty, icon, input, menu, motion, sizes, tokens};
use nsh_base::{
    DiffIndicators, DiffInline, DiffLayout, DiffViewStyles, MarkdownOptions, SubmenuEvent,
    TextViewClasses, diff_view, text_view, text_view::PartStyle, theme,
};
use nsh_core::{
    LineHeight, PropertyValue, StyleKey,
    element::{Style, desktop::title_bar},
    prelude::*,
};

use crate::llm;

/// A tick's bar height, and the padding that fills its pitch: adjacent ticks'
/// hit areas touch, so sliding along the column never leaves it between them.
const TICK_HEIGHT: f32 = 2.0;
const TICK_PAD: f32 = 4.0;

/// One session's conversation. The caller keys it by the session, so
/// switching conversations resets the draft and the scroll position.
pub(crate) fn pane(session_id: u32) -> View {
    let draft = signal(String::new());
    // The messages whose thinking the reader opened; closed again, a thought is
    // one quiet row. A reply read with its thought open starts in it.
    let open = signal(store::open_thoughts(session_id));
    // What each call shows, once the reader decided it: a call they have not
    // decided on shows its output where the mock data opened it, and its diff
    // whenever it changed files.
    let calls = signal(
        store::open_calls(session_id)
            .into_iter()
            .map(|call| CallFold { call, open: true })
            .collect::<Vec<_>>(),
    );
    // The timeline's handle: the rail's ticks scroll it to their message.
    let timeline = element_ref();
    // The reader's place: the last user message the list shows.
    let current = signal(store::last_user_index(session_id, 0, usize::MAX));
    let enter = motion().entering(motion::fade());

    view()
        .flex_1()
        .min_w(px(0.0))
        .bg(move || theme().background)
        .bind_motion(&enter)
        .child(header(session_id))
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
                        view()
                            .w_full()
                            .max_w(px(sizes::COMPOSER_MAX))
                            .min_h(px(0.0))
                            .px(px(sizes::COMPOSER_SIDE))
                            .child(timeline_view(session_id, open, calls, &timeline, current)),
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

fn header(session_id: u32) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_6()
        .pr(move || Length::from(title_bar().right + px(24.0)))
        .py_4()
        .window_drag(true)
        .border_b_1()
        .border_b_color(move || theme().border)
        .child(
            text(move || {
                store::session_summary(session_id)
                    .map(|session| session.title)
                    .unwrap_or_else(|| store::UNTITLED.into())
            })
            .flex_1()
            .min_w(px(0.0))
            .text_sm()
            .number_of_lines(1)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(move || theme().foreground),
        )
}

fn timeline_view(
    session_id: u32,
    open: Signal<Vec<(u32, usize)>>,
    calls: Signal<Vec<CallFold>>,
    handle: &ElementRef,
    current: Signal<Option<usize>>,
) -> impl IntoElement {
    virtual_list(
        move || store::message_count(session_id),
        move |at| message_row(session_id, open, calls, at),
    )
    .keys(move || store::message_ids(session_id))
    .element_ref(handle)
    .alignment(ListAlignment::Bottom)
    .scrollbar(false)
    .content_padding_x(px(sizes::CARD_PADDING))
    .content_padding_y(px(24.0))
    .estimated_item_size(px(96.0))
    .row_gap(px(16.0))
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

/// The column of question ticks at the pane's right edge: one per user message,
/// all light and short at rest. The lit ones — under the pointer, and the last
/// question the list shows — are dark and long; a press jumps to it.
fn rail(session_id: u32, timeline: ElementRef, current: Signal<Option<usize>>) -> impl IntoElement {
    // Which question's card shows, as (its row among the ticks, its message
    // index): one card for the whole column, so moving from tick to tick swaps
    // its content instead of dropping it for a frame.
    let hovered = signal(None::<(usize, usize)>);
    let height = signal(0.0f32);

    view()
        .w(px(sizes::RAIL))
        .pr(px(sizes::RAIL_INSET))
        .flex_none()
        .flex_col()
        .items_center()
        .justify_center()
        .on_mouse_leave(move |_: &MouseEvent| hovered.set(None))
        .on_layout(move |bounds: &Bounds<Points>| height.set(bounds.size.height.0))
        .child(each(
            move || store::user_message_indices(session_id),
            |index| *index,
            move |index, row| tick(index, row, timeline, current, hovered),
        ))
        .child(when(
            move || hovered.get().is_some(),
            move || message_preview(session_id, hovered, height),
        ))
}

/// One question's tick.
fn tick(
    at: ReadSignal<usize>,
    row: ReadSignal<usize>,
    timeline: ElementRef,
    current: Signal<Option<usize>>,
    hovered: Signal<Option<(usize, usize)>>,
) -> impl IntoElement {
    // Lit: the tick under the pointer, and the reader's place in the list.
    let lit = move || {
        hovered.get().is_some_and(|(_, index)| index == at.get()) || current.get() == Some(at.get())
    };

    let jump = move || {
        timeline.scroll_to_index(at.get() as u32, percentage(0.0), Points::ZERO, true);
    };

    button::bare()
        .accessibility_label(move || {
            Some(SharedString::from(format!(
                "Jump to question {}",
                row.get() + 1
            )))
        })
        .w_full()
        .items_end()
        // The hit area fills the pitch, so sliding along the column never
        // leaves it between two ticks.
        .py(px(TICK_PAD))
        .on_mouse_enter(move |_: &MouseEvent| hovered.set(Some((row.get(), at.get()))))
        .cursor_pointer()
        .on_press(jump)
        .child(
            // Every tick rests light and short; the lit ones — under the
            // pointer, or where the reader is — are dark and long.
            view()
                .h(px(TICK_HEIGHT))
                .w(move || px(if lit() { 18.0 } else { 10.0 }))
                .rounded_full()
                .bg(move || {
                    if lit() {
                        theme().primary_foreground
                    } else {
                        theme().border
                    }
                })
                .transition_all()
                .duration(motion::control_duration()),
        )
}

/// The card beside the column: the question under the pointer, then what it was
/// answered. It keeps its place in the tree while the pointer moves from tick to
/// tick, and only its content and offset change.
fn message_preview(
    session_id: u32,
    hovered: Signal<Option<(usize, usize)>>,
    height: Signal<f32>,
) -> impl IntoElement {
    let question = computed(move || {
        hovered
            .get()
            .and_then(|(_, index)| store::message_at(session_id, index))
    });
    let reply = computed(move || {
        hovered
            .get()
            .and_then(|(_, index)| store::reply_at(session_id, index))
    });

    view()
        .absolute()
        // The card ends clear of the column and a little more: the lit tick is
        // the widest of them, reaching the rail's left edge, and would show
        // under the card's own edge.
        .right(px(sizes::RAIL + 6.0))
        .top(move || {
            let (row, _) = hovered.get().unwrap_or((0, 0));
            let rows = store::user_message_count(session_id) as f32;
            let pitch = TICK_HEIGHT + TICK_PAD * 2.0;
            let start = (height.get() - rows * pitch) / 2.0;
            px(start + row as f32 * pitch - 20.0)
        })
        .w(px(320.0))
        .p_3()
        .gap_2()
        .rounded(move || theme().radius)
        .border_1()
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .shadow(tokens::floating_shadow())
        .child(
            text(body_of(question))
                .text_sm()
                .number_of_lines(3)
                .text_color(move || theme().foreground),
        )
        .child(when(
            move || reply.with(|reply| reply.is_some()),
            move || {
                // The answer sits one point smaller and a shade quieter than
                // the question it belongs to.
                text(body_of(reply))
                    .text_size(px(13.0))
                    .number_of_lines(4)
                    .text_color(move || tokens::meta().opacity(0.55))
            },
        ))
}

/// What `body` says, as the preview's text binds it.
fn body_of(body: Memo<Option<Message>>) -> impl Fn() -> SharedString + Copy + 'static {
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
fn message_row(
    session_id: u32,
    open: Signal<Vec<(u32, usize)>>,
    calls: Signal<Vec<CallFold>>,
    at: Position,
) -> impl IntoElement {
    let message = computed(move || store::message_at(session_id, at.index()));
    let from_user =
        move || message.with(|message| message.as_ref().is_some_and(|m| m.author == Author::User));

    view().w_full().child(
        when(from_user, move || user_message(message))
            .otherwise(move || assistant_message(message, open, calls)),
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

fn assistant_message(
    message: Memo<Option<Message>>,
    open: Signal<Vec<(u32, usize)>>,
    calls: Signal<Vec<CallFold>>,
) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        // The reply in the order it happened: what it thought, the call it
        // made, what it answered, and so on, each where it came.
        .child(each(
            move || {
                message.with(|message| {
                    message
                        .as_ref()
                        .map(|message| {
                            message
                                .parts
                                .iter()
                                .cloned()
                                .enumerate()
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                })
            },
            |(at, _)| *at,
            move |part, _| part_of(message, part, open, calls),
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

/// One stretch of the reply, drawn by what it is: a thought, a call, or a
/// stretch of what it answered.
fn part_of(
    message: Memo<Option<Message>>,
    part: ReadSignal<(usize, Part)>,
    open: Signal<Vec<(u32, usize)>>,
    calls: Signal<Vec<CallFold>>,
) -> impl IntoElement {
    let at = part.with_untracked(|(at, _)| *at);

    match part.with_untracked(|(_, part)| part.clone()) {
        Part::Thought(_) => thought(message, at, open).into_element(),
        Part::Text(_) => text_part(message, at).into_element(),
        Part::Tool(_) => {
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

            tool_run(tool, calls).into_element()
        }
    }
}

/// A stretch of the reply's answer, read as Markdown, the last one while it
/// still arrives as one being written.
fn text_part(message: Memo<Option<Message>>, at: usize) -> impl IntoElement {
    text_view()
        .markdown(move || {
            message.with(|message| {
                message
                    .as_ref()
                    .and_then(|message| match message.parts.get(at) {
                        Some(Part::Text(text)) => Some(text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default()
            })
        })
        .classes(TextViewClasses {
            link: Some("text-link".into()),
            ..TextViewClasses::default()
        })
        // While the stretch is still arriving, the text view shows its open
        // syntax as closed and again only the blocks that changed.
        .options(move || MarkdownOptions {
            streaming: message.with(|message| {
                message
                    .as_ref()
                    .is_some_and(|message| message.streaming && message.parts.len() == at + 1)
            }),
            ..MarkdownOptions::default()
        })
        .selectable(true)
        .code_block_scrollbar(false)
        .gap_2()
        .text_sm()
        .line_height(LineHeight::Multiple(MESSAGE_LINE))
        .text_color(move || theme().foreground)
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

/// One thought of the reply: its text under the quiet row that folds it away,
/// shown while it arrives and while the reader keeps it open. A part nothing
/// reasoned in shows nothing.
fn thought(
    message: Memo<Option<Message>>,
    at: usize,
    open: Signal<Vec<(u32, usize)>>,
) -> impl IntoElement {
    let key = move || message.with(|message| message.as_ref().map(|message| (message.id, at)));
    let shown = move || {
        let opened = key().is_some_and(|key| open.with(|open| open.contains(&key)));

        opened || writing(message, at)
    };

    when(
        move || has_thought(message, at),
        move || {
            view()
                .w_full()
                .gap_1()
                .child(thinking_bar(key, open))
                .child(when(shown, move || thinking_text(message, at)))
        },
    )
}

/// A call the reader opened or folded away.
#[derive(Clone, PartialEq)]
struct CallFold {
    /// The call.
    call: SharedString,
    /// Whether it shows what it returned.
    open: bool,
}

/// One call: the tool, the arguments it was given and how it went, over what
/// it returned — shown once the reader opens it, and from the start while it
/// changed files, so an edit's diff reads at once. The output stops at a dozen
/// lines; the agent read the whole of it.
fn tool_run(tool: Memo<ToolRun>, calls: Signal<Vec<CallFold>>) -> impl IntoElement {
    let id = move || tool.with(|tool| tool.id.clone());
    let shown = move || {
        let id = id();

        calls
            .with(|calls| {
                calls
                    .iter()
                    .find(|fold| fold.call == id)
                    .map(|fold| fold.open)
            })
            .unwrap_or_else(|| tool.with(|tool| !tool.changes.is_empty()))
    };
    let toggle = move || {
        let id = id();
        let open = !shown();

        calls.update(
            |calls| match calls.iter_mut().find(|fold| fold.call == id) {
                Some(fold) => fold.open = open,
                None => calls.push(CallFold { call: id, open }),
            },
        );
    };

    view()
        .w_full()
        .gap_1()
        .child(
            button::bare()
                .w_full()
                .group()
                .flex_row()
                .items_center()
                .gap_2()
                .px_1()
                .py(px(2.0))
                .text_color(tokens::meta)
                .on_press(toggle)
                .child(icon::wrench(14.0))
                .child(
                    text(move || tool.with(|tool| tool.name.clone()))
                        .flex_none()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view().flex_1().min_w(px(0.0)).child(
                        // An edit names the file it changed and how far, where a
                        // call that only runs something previews its arguments.
                        when(
                            move || tool.with(|tool| !tool.changes.is_empty()),
                            move || edit_summary(tool),
                        )
                        .otherwise(move || {
                            text(move || tool.with(|tool| tool.arguments.clone()))
                                .w_full()
                                .number_of_lines(1)
                                .font_family(crate::MONO_FONT)
                                .text_size(px(sizes::CODE_FONT))
                                .text_color(tokens::meta)
                        }),
                    ),
                )
                .child(when(
                    move || tool.with(|tool| tool.running),
                    || text("running…").text_xs().text_color(tokens::meta),
                ))
                .child(when(
                    move || tool.with(|tool| tool.failed),
                    || text("failed").text_xs().text_color(tokens::caption),
                ))
                .child(
                    // The chevron is what says a press opens the call's output:
                    // it shows while the pointer is over the row, and for the
                    // keyboard's focus, leaving the reply quiet at rest.
                    view()
                        .opacity(0.0)
                        .group_hover(|s| s.opacity(1.0))
                        .group_focus_within(|s| s.opacity(1.0))
                        .transition_opacity()
                        .duration(motion::row_duration())
                        .ease_out()
                        .child(
                            when(shown, || icon::chevron_up(14.0))
                                .otherwise(|| icon::chevron_down(14.0)),
                        ),
                ),
        )
        .child(when(
            move || shown() && tool.with(|tool| !tool.changes.is_empty()),
            move || changes(tool),
        ))
        .child(when(
            move || shown() && tool.with(|tool| tool.changes.is_empty() && !tool.output.is_empty()),
            move || {
                view()
                    .w_full()
                    .rounded(move || theme().radius)
                    .bg(tokens::code)
                    .px_2()
                    .py_1()
                    // A blank line under the last one, as tall as the text: the
                    // result ends where it did, not against the block's edge.
                    .pb(px(sizes::CODE_FONT))
                    .child(
                        text(move || tool.with(|tool| tool.output.clone()))
                            .w_full()
                            .font_family(crate::MONO_FONT)
                            .text_size(px(sizes::CODE_FONT))
                            .line_height(LineHeight::Multiple(MESSAGE_LINE))
                            .text_color(tokens::meta)
                            .number_of_lines(12),
                    )
            },
        ))
}

/// What an edit did, where the row names the call: the first file it changed
/// and how far the call changed what it touched, in all.
fn edit_summary(tool: Memo<ToolRun>) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            text(move || {
                tool.with(|tool| {
                    tool.changes
                        .first()
                        .map(|change| change.path.clone())
                        .unwrap_or_default()
                })
            })
            .flex_1()
            .min_w(px(0.0))
            .number_of_lines(1)
            .font_family(crate::MONO_FONT)
            .text_size(px(sizes::CODE_FONT))
            .text_color(tokens::meta),
        )
        .child(
            text(move || {
                tool.with(|tool| {
                    let added: usize = tool.changes.iter().map(|change| change.added).sum();
                    let removed: usize = tool.changes.iter().map(|change| change.removed).sum();

                    format!("+{added} −{removed}")
                })
            })
            .flex_none()
            .font_family(crate::MONO_FONT)
            .text_size(px(sizes::CODE_FONT))
            .text_color(tokens::meta),
        )
}

/// How tall one row of a diff is, in points.
const DIFF_ROW: f32 = 14.0;

/// How many rows a diff shows before it scrolls its own: a change longer than
/// this is read through the view rather than pushing the reply down.
const DIFF_ROWS: f32 = 24.0;

/// The files a call changed: each a card of its own, under a blank line that
/// keeps it off the row that names the call.
fn changes(tool: Memo<ToolRun>) -> impl IntoElement {
    view().w_full().gap_2().pt_1().child(each(
        move || tool.with(|tool| tool.changes.clone()),
        |change| change.path.clone(),
        move |change, _| change_card(change),
    ))
}

/// One file a call changed: its path and how far the call changed it, over the
/// rows of its diff, in one card.
fn change_card(change: ReadSignal<store::ToolChange>) -> impl IntoElement {
    view()
        .w_full()
        .rounded(move || theme().radius)
        // The rows' fills are square: the card's corners keep them.
        .overflow_hidden()
        .bg(tokens::code)
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap_2()
                // The bar the view used to put over a change's first row; the
                // file's name takes its place.
                .bg(move || theme().muted)
                .px_2()
                .py_1()
                .child(
                    text(move || change.with(|change| change.path.clone()))
                        .flex_1()
                        .min_w(px(0.0))
                        .number_of_lines(1)
                        .font_family(crate::MONO_FONT)
                        .text_size(px(sizes::CODE_FONT))
                        .text_color(tokens::meta),
                )
                .child(
                    text(move || {
                        change.with(|change| format!("+{} −{}", change.added, change.removed))
                    })
                    .flex_none()
                    .font_family(crate::MONO_FONT)
                    .text_size(px(sizes::CODE_FONT))
                    .text_color(tokens::meta),
                ),
        )
        // A file too large for the tool to diff has its counts alone.
        .child(when(
            move || change.with(|change| !change.diff.is_empty()),
            move || diff_rows(change),
        ))
}

/// The rows of one file's change, drawn by the diff view: the rows it changed
/// and the words changed within them, in the theme's own fills.
///
/// The view is given the two texts rather than the patch: a patch carries a
/// header for each of its hunks, and the view draws one as a row of its own — a
/// band over the first change, which a card that names no lines has no use for.
/// The two texts hold the hunks' lines alone, so every row is a line of them.
///
/// The view lays its rows out absolutely and fills the box it is given, so the
/// card gives it the height they take — counted from the patch, its header
/// lines aside — up to [`DIFF_ROWS`], past which the view scrolls them.
fn diff_rows(change: ReadSignal<store::ToolChange>) -> impl IntoElement {
    let sides = computed(move || change.with(|change| sides_of(&change.diff)));
    let rows = move || {
        change.with(|change| {
            change
                .diff
                .lines()
                .filter(|line| {
                    !line.starts_with("--- ")
                        && !line.starts_with("+++ ")
                        && matches!(line.as_bytes().first(), Some(b' ' | b'-' | b'+'))
                })
                .count()
                .max(1) as f32
        })
    };

    view()
        .w_full()
        .h(move || px(rows().min(DIFF_ROWS) * DIFF_ROW))
        .child(
            diff_view()
                .old_value(move || sides.with(|sides| Some(sides.before.clone())))
                .new_value(move || sides.with(|sides| Some(sides.after.clone())))
                // The hunks' own lines are the whole of what reads: a line
                // kept around a change is in them already.
                .context_lines(0)
                .layout(DiffLayout::Unified)
                .inline(DiffInline::Word)
                // The view's own fills are its green and red; the card's
                // are the theme's, and no second one under them.
                .background(false)
                // The card names no lines: a change in a reply is read for what
                // it did, not for where in the file it sits.
                .line_numbers(false)
                // Its fills say what changed; a bar beside them says it again.
                .indicators(DiffIndicators::None)
                // Every row shows, so the height counted above is the
                // height the rows take.
                .expand_all(true)
                .font_size(px(sizes::CODE_FONT))
                .line_height(px(DIFF_ROW))
                .styles(diff_styles)
                // The view fills the box the card measured for it.
                .size_full(),
        )
}

/// The two texts `patch` compares: what the file held and what it holds, each
/// of the hunks' lines in order, the untouched lines between hunks left out
/// with the headers that name them.
#[derive(PartialEq)]
struct Sides {
    before: SharedString,
    after: SharedString,
}

fn sides_of(patch: &str) -> Sides {
    let mut before = String::new();
    let mut after = String::new();
    for line in patch.lines() {
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            continue;
        }
        let Some((sign, text)) = line.split_at_checked(1) else {
            continue;
        };
        match sign {
            " " => {
                before.push_str(text);
                before.push('\n');
                after.push_str(text);
                after.push('\n');
            }
            "-" => {
                before.push_str(text);
                before.push('\n');
            }
            "+" => {
                after.push_str(text);
                after.push('\n');
            }
            // A hunk's header, and the note a last line without its own ending
            // leaves: neither is a line of either text.
            _ => {}
        }
    }

    Sides {
        before: before.into(),
        after: after.into(),
    }
}

/// The diff's fills: the theme's own, over the view's green and red.
fn diff_styles() -> DiffViewStyles {
    let mut styles = DiffViewStyles::default();
    styles.line_added = Some(fill(tokens::diff_added()));
    styles.line_deleted = Some(fill(tokens::diff_removed()));
    styles.word_added = Some(fill(tokens::diff_added_word()));
    styles.word_deleted = Some(fill(tokens::diff_removed_word()));
    // A stretch of lines the view leaves out is not a row of this card: the
    // rows it shows are the whole of the change.
    styles.separator = Some(faded());

    styles
}

/// A part that takes its row but shows nothing of it.
fn faded() -> PartStyle {
    PartStyle(Style::default().with(StyleKey::Opacity, PropertyValue::Number(0.0.into())))
}

/// `color` as a part's fill, in the appearance now: the view's own fills are
/// styles on its elements, which carry both appearances, while a part's style
/// is a patch of declarations, which carries one color.
fn fill(color: AdaptiveColor) -> PartStyle {
    PartStyle(Style::default().with(
        StyleKey::BackgroundColor,
        PropertyValue::Color(Some(nsh_core::element::color(color).into())),
    ))
}

/// The row a thought folds into: a bulb and one word at the left, and the
/// chevron that opens it at the right, whose own wash on hover keeps the row
/// itself quiet.
fn thinking_bar(
    key: impl Fn() -> Option<(u32, usize)> + Copy + 'static,
    open: Signal<Vec<(u32, usize)>>,
) -> impl IntoElement {
    let toggle = move || {
        let Some(key) = key() else {
            return;
        };
        open.update(|open| match open.iter().position(|shown| *shown == key) {
            Some(at) => {
                open.remove(at);
            }
            None => open.push(key),
        });
    };
    let opened = move || key().is_some_and(|key| open.with(|open| open.contains(&key)));

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
        .child(
            button::ghost_icon(
                when(opened, || icon::chevron_up(14.0)).otherwise(|| icon::chevron_down(14.0)),
                "Expand or collapse the thinking",
            )
            // The chevron shows while the pointer is over the thought's row,
            // and for the keyboard's focus, as a call's does.
            .opacity(0.0)
            .group_hover(|s| s.opacity(1.0))
            .group_focus_within(|s| s.opacity(1.0))
            .on_press(toggle),
        )
}

/// Where a thought's text starts: under the word beside its bulb, past the
/// row's inset, the mark and the gap between them.
const THINKING_TEXT_INSET: f32 = 26.0;

/// The thought's own text size, and the blank line kept under it: one line of
/// that text, so the answer opens a line below the thought.
const THINKING_FONT: f32 = 13.0;
const THINKING_LINE: f32 = THINKING_FONT * MESSAGE_LINE;

/// The thought's own text, quieter than the answer it led to.
fn thinking_text(message: Memo<Option<Message>>, at: usize) -> impl IntoElement {
    text(move || thought_text(message, at).unwrap_or_default())
        .w_full()
        .pl(px(THINKING_TEXT_INSET))
        .pb(px(THINKING_LINE))
        .text_size(px(THINKING_FONT))
        .line_height(LineHeight::Multiple(MESSAGE_LINE))
        .text_color(move || tokens::meta().opacity(0.7))
}

fn composer(session_id: u32, draft: Signal<String>) -> impl IntoElement {
    let can_send = move || draft.with(|draft| !draft.trim().is_empty());
    let submit = move || {
        let text = draft.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        llm::ask(session_id, &text);
        draft.set(String::new());
    };
    // Opening a conversation puts the reader in the composer: the field takes
    // the keys once it is on screen.
    let field = element_ref();
    set_timeout(Duration::from_millis(50), move || field.focus());

    view()
        .pb_2()
        .gap_2()
        .child(composer_card(draft, can_send, submit, field))
        // One line reserved for the status bar; it holds no text yet.
        .child(view().h_5().pl(px(sizes::CARD_PADDING)).justify_center())
}

/// The floating composer: a rounded card whose input sits on top and whose
/// round send button sits in the toolbar row below. The card is the frame:
/// the input itself is borderless, and the card's own edge is a whisper with a
/// shadow under it, not a ring that lights up.
fn composer_card(
    draft: Signal<String>,
    can_send: impl Fn() -> bool + Copy + 'static,
    submit: impl Fn() + Copy + 'static,
    field: ElementRef,
) -> impl IntoElement {
    view()
        .p(px(sizes::CARD_PADDING))
        .gap_2()
        .rounded_3xl()
        .border_1()
        .border_color(move || theme().border.opacity(0.6))
        .bg(move || theme().card)
        .shadow(tokens::floating_shadow())
        .child(
            input::composer(draft, submit)
                .label("Message")
                .element_ref(&field),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_end()
                .gap_2()
                .child(model_picker())
                .child(button::icon("↑", "Send", can_send).on_press(submit)),
        )
}

/// The model and the effort the next reply asks: each row names what it holds,
/// and the list it opens holds the choices.
fn model_picker() -> impl IntoElement {
    let query = signal(String::new());
    // The field that takes the keys as the list opens.
    let search = element_ref();
    // The menu's rows are plain presses, which leave the search's keys alone:
    // the app closes it when one is chosen.
    let open = signal(false);

    menu::controlled(
        open,
        menu::label_trigger(
            badge::tag(move || settings::selected().provider_name),
            move || settings::selected().label,
            move || SharedString::from(effort_label(settings::effort())),
        )
        .disabled(move || settings::enabled_models().is_empty()),
        Placement::Top,
        Align::End,
        view()
            .child(
                menu::submenu(
                    "Model",
                    move || settings::selected().label,
                    view()
                        .min_w(px(240.0))
                        .gap_1()
                        // The row that opened it takes no focus (a pointer
                        // over it would take the field's), so the keys stay
                        // in the field as the list is searched.
                        .child(
                            menu::search(query, "Search models…")
                                .auto_focus(true)
                                .element_ref(&search),
                        )
                        // The models scroll in their own view under the field,
                        // so a wheel over them never takes the field's keys.
                        .child(
                            scroll()
                                .max_h(px(360.0))
                                .scrollbar(false)
                                .child(model_choices(query, open)),
                        ),
                )
                .on_open_change(move |event: &SubmenuEvent| {
                    let SubmenuEvent::OpenChange(open, _) = event;
                    if *open {
                        // The popup is made in this commit: the field takes
                        // the keys once it is there.
                        set_timeout(Duration::from_millis(50), move || search.focus());
                    } else {
                        // Each opening searches from the start again.
                        query.set(String::new());
                    }
                }),
            )
            .child(menu::submenu(
                "Effort",
                move || SharedString::from(effort_label(settings::effort())),
                effort_choices(),
            )),
    )
}

/// The models the settings offer, grouped under the provider that serves them
/// and narrowed to `query` (every one while it is empty), the chosen one
/// checked.
fn model_choices(query: Signal<String>, open: Signal<bool>) -> impl IntoElement {
    each(
        move || {
            let query = query.with(|query| query.trim().to_lowercase());
            let mut sections: Vec<ModelSection> = Vec::new();
            for choice in settings::enabled_models() {
                if !query.is_empty() && !choice.label.as_str().to_lowercase().contains(&query) {
                    continue;
                }
                match sections.last_mut() {
                    Some(section) if section.provider == choice.provider => {
                        section.choices.push(choice);
                    }
                    _ => sections.push(ModelSection {
                        provider: choice.provider,
                        name: choice.provider_name.clone(),
                        choices: vec![choice],
                    }),
                }
            }
            sections
        },
        |section| section.provider,
        move |section, _| {
            view()
                .w_full()
                .gap_1()
                .child(menu::heading(move || {
                    section.with(|section| section.name.clone())
                }))
                .child(each(
                    move || section.with(|section| section.choices.clone()),
                    |choice| choice.id.clone(),
                    move |choice, _| {
                        menu::option(
                            move || choice.with(|choice| choice.label.clone()),
                            move || {
                                let selected = settings::selected();
                                choice.with(|choice| {
                                    choice.provider == selected.provider && choice.id == selected.id
                                })
                            },
                        )
                        .on_press(move || {
                            let (provider, id) =
                                choice.with(|choice| (choice.provider, choice.id.clone()));
                            settings::select_model(provider, id.as_str());
                            open.set(false);
                        })
                    },
                ))
        },
    )
}

/// The models one provider serves, as the picker lists them under its name.
#[derive(Clone, PartialEq)]
struct ModelSection {
    provider: u32,
    name: SharedString,
    choices: Vec<settings::Choice>,
}

/// The efforts a reply can ask for, the weakest first.
fn effort_choices() -> impl IntoElement {
    view()
        .min_w(px(160.0))
        .gap_1()
        .child(effort_choice(Effort::Default))
        .child(effort_choice(Effort::Low))
        .child(effort_choice(Effort::Medium))
        .child(effort_choice(Effort::High))
}

fn effort_choice(effort: Effort) -> impl IntoElement {
    menu::choice(effort_label(effort), move || settings::effort() == effort)
        .on_select(move || settings::select_effort(effort))
}

/// What `effort` is called in the picker.
fn effort_label(effort: Effort) -> &'static str {
    match effort {
        Effort::Default => "Default",
        Effort::Low => "Low",
        Effort::Medium => "Medium",
        Effort::High => "High",
    }
}
