//! The conversation: its header, its message timeline and its composer.

use std::time::Duration;

use aedt_provider::responses::Effort;
use aedt_settings as settings;
use aedt_store::{self as store, Author, Message, ToolRun};
use aedt_ui::{MESSAGE_LINE, button, empty, icon, input, menu, motion, sizes, tokens};
use nsh_base::{MarkdownOptions, SubmenuEvent, TextViewClasses, hover_card, text_view, theme};
use nsh_core::{LineHeight, element::desktop::title_bar, prelude::*};

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
    // The tool calls whose output the reader opened, by the call's id.
    let open_calls = signal(store::open_calls(session_id));
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
                            .child(timeline_view(
                                session_id, open, open_calls, &timeline, current,
                            )),
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
    open_calls: Signal<Vec<SharedString>>,
    handle: &ElementRef,
    current: Signal<Option<usize>>,
) -> impl IntoElement {
    virtual_list(
        move || store::message_count(session_id),
        move |at| message_row(session_id, open, open_calls, at),
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
        .right(px(sizes::RAIL - 6.0))
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
        body.with(|body| {
            body.as_ref()
                .map(|body| body.body.clone())
                .unwrap_or_default()
        })
    }
}

/// A cell, made once and given other messages as it is reused.
fn message_row(
    session_id: u32,
    open: Signal<Vec<(u32, usize)>>,
    open_calls: Signal<Vec<SharedString>>,
    at: Position,
) -> impl IntoElement {
    let message = computed(move || store::message_at(session_id, at.index()));
    let from_user =
        move || message.with(|message| message.as_ref().is_some_and(|m| m.author == Author::User));

    view().w_full().child(
        when(from_user, move || user_message(message))
            .otherwise(move || assistant_message(message, open, open_calls)),
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
                                    .map(|message| message.body.clone())
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
                .map(|message| message.body.to_string())
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
    open_calls: Signal<Vec<SharedString>>,
) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        // The reply as it happened: what it thought, the call it made, what it
        // thought next, and so on — its answer's text under them all.
        .child(thought(message, || 0, open))
        .child(each(
            move || {
                message.with(|message| {
                    message
                        .as_ref()
                        .map(|message| message.tools.clone())
                        .unwrap_or_default()
                })
            },
            |tool| tool.id.clone(),
            move |tool, call| {
                view()
                    .w_full()
                    .gap_1()
                    .child(tool_run(tool, open_calls))
                    // The thought that followed the call.
                    .child(thought(message, move || call.get() + 1, open))
            },
        ))
        .child(
            text_view()
                .markdown(move || {
                    message.with(|message| message.as_ref().map(|message| message.body.clone()))
                })
                .classes(TextViewClasses {
                    link: Some("text-link".into()),
                    ..TextViewClasses::default()
                })
                // While the reply is still arriving, the text view shows its
                // open syntax as closed and again only the blocks that changed.
                .options(move || MarkdownOptions {
                    streaming: message
                        .with(|message| message.as_ref().is_some_and(|message| message.streaming)),
                    ..MarkdownOptions::default()
                })
                .selectable(true)
                .code_block_scrollbar(false)
                .gap_2()
                .text_sm()
                .line_height(LineHeight::Multiple(MESSAGE_LINE))
                .text_color(move || theme().foreground),
        )
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

/// What the thought of `slot` says: the entry before the call at its place,
/// and after the last call everything the reply thought since — one entry per
/// call, but a reply's first thought may hold more than one paragraph.
fn thought_text(message: &Message, slot: usize) -> String {
    if slot < message.tools.len() {
        return message
            .reasoning
            .get(slot)
            .map_or_else(String::new, |thought| thought.trim_end().to_owned());
    }

    message
        .reasoning
        .iter()
        .skip(slot)
        .map(SharedString::as_str)
        .filter(|thought| !thought.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
        .trim_end()
        .to_owned()
}

/// Whether the thought of `slot` has arrived.
fn has_thought(message: Memo<Option<Message>>, slot: impl Fn() -> usize + Copy + 'static) -> bool {
    message.with(|message| {
        message
            .as_ref()
            .is_some_and(|message| !thought_text(message, slot()).is_empty())
    })
}

/// Whether the thought of `slot` is the one arriving: the reply streams and
/// its answer has not begun.
fn writing(message: Memo<Option<Message>>, slot: impl Fn() -> usize + Copy + 'static) -> bool {
    message.with(|message| {
        message.as_ref().is_some_and(|message| {
            message.streaming && message.body.is_empty() && slot() == message.tools.len()
        })
    })
}

/// One thought of the reply: its text under the quiet row that folds it away,
/// shown while it arrives and while the reader keeps it open. A slot nothing
/// reasoned in shows nothing.
fn thought(
    message: Memo<Option<Message>>,
    slot: impl Fn() -> usize + Copy + 'static,
    open: Signal<Vec<(u32, usize)>>,
) -> impl IntoElement {
    let key = move || message.with(|message| message.as_ref().map(|message| (message.id, slot())));
    let shown = move || {
        let opened = key().is_some_and(|key| open.with(|open| open.contains(&key)));

        opened || writing(message, slot)
    };

    when(
        move || has_thought(message, slot),
        move || {
            view()
                .w_full()
                .gap_1()
                .child(thinking_bar(key, open))
                .child(when(shown, move || thinking_text(message, slot)))
        },
    )
}

/// One call: the tool, the arguments it was given and how it went, over what
/// it returned — shown once the reader opens it. The output stops at a dozen
/// lines; the agent read the whole of it.
fn tool_run(tool: ReadSignal<ToolRun>, open: Signal<Vec<SharedString>>) -> impl IntoElement {
    let id = move || tool.with(|tool| tool.id.clone());
    let shown = move || {
        let id = id();
        open.with(|open| open.contains(&id))
    };
    let toggle = move || {
        let id = id();
        open.update(|open| match open.iter().position(|shown| *shown == id) {
            Some(at) => {
                open.remove(at);
            }
            None => open.push(id),
        });
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
                        hover_card()
                            .placement(Placement::Top)
                            .align(Align::Start)
                            .offset(px(6.0))
                            .open_delay(600.0)
                            .popup_style(|s| {
                                s.p_2()
                                    .rounded(move || theme().radius)
                                    .border_1()
                                    .border_color(move || theme().border)
                                    .bg(move || theme().card)
                                    .shadow(tokens::floating_shadow())
                            })
                            .trigger(
                                text(move || tool.with(|tool| tool.arguments.clone()))
                                    .w_full()
                                    .number_of_lines(1)
                                    .font_family(crate::MONO_FONT)
                                    .text_size(px(sizes::CODE_FONT))
                                    .text_color(tokens::meta),
                            )
                            .child(arguments_card(tool)),
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
            move || shown() && tool.with(|tool| !tool.output.is_empty()),
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

/// The arguments a call was given, the whole of them, read as JSON: over the
/// line that previews them while the pointer rests on it.
fn arguments_card(tool: ReadSignal<ToolRun>) -> impl IntoElement {
    scroll()
        .max_h(px(420.0))
        .child(
            view()
                .min_w(px(320.0))
                .max_w(px(640.0))
                .child(text_view().markdown(move || -> SharedString {
                    tool.with(|tool| {
                        let arguments = tool.arguments.as_str();
                        let pretty = serde_json::from_str::<serde_json::Value>(arguments)
                            .ok()
                            .and_then(|value| serde_json::to_string_pretty(&value).ok())
                            .unwrap_or_else(|| arguments.to_owned());
                        format!("```json\n{pretty}\n```").into()
                    })
                })),
        )
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
fn thinking_text(
    message: Memo<Option<Message>>,
    slot: impl Fn() -> usize + Copy + 'static,
) -> impl IntoElement {
    text(move || {
        message.with(|message| {
            message
                .as_ref()
                .map(|message| thought_text(message, slot()))
                .unwrap_or_default()
        })
    })
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
