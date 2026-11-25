//! The conversation: its header, its message timeline and its composer.

use nsh_base::{MarkdownOptions, TextViewClasses, text_view, theme};
use nsh_core::{LineHeight, element::desktop::title_bar, prelude::*};

use crate::{
    store::{self, Author, Message},
    ui::{self, badge, button, empty, icon, input, motion, sizes, tokens},
};

/// A tick's bar height, and the padding that fills its pitch: adjacent ticks'
/// hit areas touch, so sliding along the column never leaves it between them.
const TICK_HEIGHT: f32 = 2.0;
const TICK_PAD: f32 = 4.0;

/// One session's conversation. The caller keys it by the session, so
/// switching conversations resets the draft and the scroll position.
pub(crate) fn pane(session_id: u32) -> View {
    let draft = signal(String::new());
    // The messages whose thinking the reader opened; closed again, a thought is
    // one quiet row.
    let open = signal(Vec::<u32>::new());
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
                    view()
                        .flex_1()
                        .min_h(px(0.0))
                        .w_full()
                        .flex_row()
                        // Gutters as wide as the rail, so the conversation stays
                        // centered in the pane whatever its width.
                        .child(view().w(px(sizes::RAIL)).flex_none())
                        .child(
                            view()
                                .flex_1()
                                .min_w(px(0.0))
                                .min_h(px(0.0))
                                .flex_row()
                                .justify_center()
                                .child(
                                    view()
                                        .w_full()
                                        .max_w(px(sizes::COMPOSER_MAX))
                                        .min_h(px(0.0))
                                        .px(px(sizes::COMPOSER_SIDE))
                                        .child(timeline_view(session_id, open, &timeline, current)),
                                ),
                        )
                        .child(rail(session_id, timeline, current)),
                )
                .child(
                    view()
                        .w_full()
                        .flex_none()
                        .flex_row()
                        .child(view().w(px(sizes::RAIL)).flex_none())
                        .child(
                            view()
                                .flex_1()
                                .min_w(px(0.0))
                                .flex_row()
                                .justify_center()
                                .child(
                                    view()
                                        .w_full()
                                        .max_w(px(sizes::COMPOSER_MAX))
                                        .px(px(sizes::COMPOSER_SIDE))
                                        .child(composer(session_id, draft)),
                                ),
                        )
                        .child(view().w(px(sizes::RAIL)).flex_none()),
                ),
        )
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
                    .unwrap_or_else(|| "新会话".into())
            })
            .flex_1()
            .min_w(px(0.0))
            .text_sm()
            .number_of_lines(1)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(move || theme().foreground),
        )
        .child(badge::badge(crate::llm::MODEL))
}

fn timeline_view(
    session_id: u32,
    open: Signal<Vec<u32>>,
    handle: &ElementRef,
    current: Signal<Option<usize>>,
) -> impl IntoElement {
    virtual_list(
        move || store::message_count(session_id),
        move |at| message_row(session_id, open, at),
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
    .empty(empty::empty("说点什么…"))
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
        .accessibility_label("跳转到该消息")
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
fn message_row(session_id: u32, open: Signal<Vec<u32>>, at: Position) -> impl IntoElement {
    let message = computed(move || store::message_at(session_id, at.index()));
    let from_user =
        move || message.with(|message| message.as_ref().is_some_and(|m| m.author == Author::User));

    view().w_full().child(
        when(from_user, move || user_message(message))
            .otherwise(move || assistant_message(message, open)),
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
                        .line_height(LineHeight::Multiple(ui::MESSAGE_LINE))
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
        if !body.is_empty() {
            drop(set_clipboard(body));
        }
    };

    button::ghost_icon(icon::copy(14.0), "复制消息")
        .text_color(move || theme().muted_foreground)
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
                        .and_then(|message| message.meta.clone())
                        .unwrap_or_default()
                })
            })
            .text_xs()
            .text_color(tokens::caption)
        },
    )
}

fn assistant_message(message: Memo<Option<Message>>, open: Signal<Vec<u32>>) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        .child(when(
            move || has_reasoning(message),
            move || thinking(message, open),
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
                .text_size(px(14.0))
                .line_height(LineHeight::Multiple(ui::MESSAGE_LINE))
                .text_color(move || theme().foreground),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_1()
                .child(copy_message(message))
                .child(message_meta(message)),
        )
}

/// Whether the reply reasoned before answering.
fn has_reasoning(message: Memo<Option<Message>>) -> bool {
    message.with(|message| {
        message
            .as_ref()
            .is_some_and(|message| !message.reasoning.is_empty())
    })
}

/// The chain of thought above the answer: its text while it arrives and while
/// the reader keeps it open, else one quiet row that opens it again.
fn thinking(message: Memo<Option<Message>>, open: Signal<Vec<u32>>) -> impl IntoElement {
    let id = move || message.with(|message| message.as_ref().map(|message| message.id));
    let shown = move || {
        let opened = id().is_some_and(|id| open.with(|open| open.contains(&id)));
        // Until the answer starts, the thought is what is being written.
        let writing = message.with(|message| {
            message
                .as_ref()
                .is_some_and(|message| message.streaming && message.body.is_empty())
        });

        opened || writing
    };

    view()
        .w_full()
        .gap_1()
        .child(thinking_bar(id, open))
        .child(when(shown, move || thinking_text(message)))
}

/// The row a thought folds into: a bulb and one word, pressed to open it.
fn thinking_bar(
    id: impl Fn() -> Option<u32> + Copy + 'static,
    open: Signal<Vec<u32>>,
) -> impl IntoElement {
    let toggle = move || {
        let Some(id) = id() else {
            return;
        };
        open.update(|open| match open.iter().position(|shown| *shown == id) {
            Some(at) => {
                open.remove(at);
            }
            None => open.push(id),
        });
    };

    button::quiet(icon::bulb(14.0), "思考")
        .accessibility_label("展开或收起思考")
        .text_color(tokens::meta)
        .on_press(toggle)
}

/// The thought itself, quieter than the answer it led to.
fn thinking_text(message: Memo<Option<Message>>) -> impl IntoElement {
    text(move || {
        message.with(|message| {
            message
                .as_ref()
                .map(|message| message.reasoning.clone())
                .unwrap_or_default()
        })
    })
    .w_full()
    .text_size(px(13.0))
    .line_height(LineHeight::Multiple(ui::MESSAGE_LINE))
    .text_color(tokens::meta)
}

fn composer(session_id: u32, draft: Signal<String>) -> impl IntoElement {
    let can_send = move || draft.with(|draft| !draft.trim().is_empty());
    let submit = move || {
        let text = draft.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        store::send(session_id, &text);
        draft.set(String::new());
    };

    view()
        .pb_2()
        .gap_2()
        .child(composer_card(draft, can_send, submit))
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
) -> impl IntoElement {
    view()
        .p(px(sizes::CARD_PADDING))
        .gap_2()
        .rounded_3xl()
        .border_1()
        .border_color(move || theme().border.opacity(0.6))
        .bg(move || theme().card)
        .shadow(tokens::floating_shadow())
        .child(input::composer(draft, submit).label("消息"))
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_end()
                .child(button::icon("↑", "发送", can_send).on_press(submit)),
        )
}
