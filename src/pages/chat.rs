//! The conversation: its header, its message timeline and its composer.

use nsh_base::{MarkdownOptions, TextViewClasses, button, text_view, theme};
use nsh_core::{Color, LineHeight, element::desktop::title_bar, prelude::*};

use crate::{
    store::{self, Author, Message},
    ui::{self, badge, button, empty, icon, input, motion, sizes},
};

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
                store::session_by_id(session_id)
                    .map(|session| session.display_title())
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
    view()
        .w(px(sizes::RAIL))
        .pr(px(sizes::RAIL_INSET))
        .flex_none()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(each(
            move || store::user_message_indices(session_id),
            |index| *index,
            move |index, _row| tick(session_id, index, timeline, current),
        ))
}

/// One question's tick.
fn tick(
    session_id: u32,
    at: ReadSignal<usize>,
    timeline: ElementRef,
    current: Signal<Option<usize>>,
) -> impl IntoElement {
    let message = computed(move || store::message_at(session_id, at.get()));
    let reply = computed(move || store::reply_after(session_id, at.get()));
    let hovered = signal(false);
    // Lit: the tick under the pointer, and the reader's place in the list.
    let lit = move || hovered.get() || current.get() == Some(at.get());

    let jump = move || {
        timeline.scroll_to_index(at.get() as u32, percentage(0.0), Points::ZERO, true);
    };

    button()
        .accessibility_label("跳转到该消息")
        .w_full()
        .items_end()
        .py(px(2.0))
        .on_mouse_enter(move |_: &MouseEvent| hovered.set(true))
        .on_mouse_leave(move |_: &MouseEvent| hovered.set(false))
        .cursor_pointer()
        .on_press(jump)
        .child(
            // Every tick rests light and short; the lit ones — under the
            // pointer, or where the reader is — are dark and long.
            view()
                .h(px(2.0))
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
        .child(when(
            move || hovered.get(),
            move || message_preview(message, reply),
        ))
}

/// The card beside a hovered tick: the question, then what it was answered.
fn message_preview(
    question: Memo<Option<Message>>,
    reply: Memo<Option<Message>>,
) -> impl IntoElement {
    view()
        .absolute()
        .right(px(sizes::RAIL - 6.0))
        .top(px(-10.0))
        .w(px(320.0))
        .p_3()
        .gap_2()
        .rounded(move || theme().radius)
        .border_1()
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .shadow(composer_shadow())
        .child(preview_part(question, 3))
        .child(when(
            move || reply.with(|reply| reply.is_some()),
            move || preview_part(reply, 4),
        ))
}

/// One side of the preview card: the start of what was said.
fn preview_part(body: Memo<Option<Message>>, lines: u32) -> impl IntoElement {
    text(move || {
        body.with(|body| {
            body.as_ref()
                .map(|body| body.body.clone())
                .unwrap_or_default()
        })
    })
    .text_sm()
    .number_of_lines(lines)
    .text_color(move || theme().foreground)
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
                    .bg(move || theme().color("bubble").unwrap_or_else(|| theme().primary))
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
            .text_color(move || {
                theme()
                    .color("meta")
                    .unwrap_or_else(|| theme().muted_foreground)
            })
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

/// The composer's shadow: two wide, faint layers, so the card seems to float
/// on haze instead of sitting inside a dark edge.
fn composer_shadow() -> Vec<BoxShadow> {
    let layer = |offset_y: f32, blur: f32, alpha: u8| BoxShadow {
        offset_x: Points(0.0),
        offset_y: Points(offset_y),
        blur: Points(blur),
        spread: Points(0.0),
        color: Color::rgba(0, 0, 0, alpha),
        inset: false,
    };
    vec![layer(1.0, 2.0, 10), layer(6.0, 20.0, 13)]
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

    button()
        .accessibility_label("展开或收起思考")
        .flex_row()
        .items_center()
        .gap_2()
        .px_1()
        .py(px(2.0))
        .rounded(move || theme().radius)
        .text_color(thinking_ink)
        .hover(|s| s.bg(move || theme().muted))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .transition_colors()
        .duration(motion::control_duration())
        .ease_out()
        .on_press(toggle)
        .child(icon::bulb(14.0))
        .child(text("思考").text_xs())
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
    .text_color(thinking_ink)
}

/// A thought's ink: the theme's `meta` (the quietest text the app has), or the
/// muted one when the theme leaves it out.
fn thinking_ink() -> AdaptiveColor {
    theme()
        .color("meta")
        .unwrap_or_else(|| theme().muted_foreground)
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
        .shadow(composer_shadow())
        .child(input::composer(draft, submit).label("消息"))
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_end()
                .child(button::icon("↑", "发送", can_send).on_press(submit)),
        )
}
