//! The conversation: its header, its message timeline and its composer.

use nsh_base::{MarkdownOptions, TextViewClasses, text_view, theme};
use nsh_core::{Color, LineHeight, element::desktop::title_bar, prelude::*};

use crate::{
    store::{self, Author, Message},
    ui::{badge, button, empty, input, motion, sizes},
};

/// A message's line height, a multiple of its font size: lines sit close, as a
/// chat log's do, rather than at a document's spacing.
const MESSAGE_LINE: f32 = 1.25;

/// One session's conversation. The caller keys it by the session, so
/// switching conversations resets the draft and the scroll position.
pub(crate) fn pane(session_id: u32) -> View {
    let draft = signal(String::new());
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
                .max_w(px(sizes::COMPOSER_MAX))
                .self_center()
                .px(px(sizes::COMPOSER_SIDE))
                .child(timeline(session_id))
                .child(composer(session_id, draft)),
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

fn timeline(session_id: u32) -> impl IntoElement {
    virtual_list(
        move || store::message_count(session_id),
        move |at| message_row(session_id, at),
    )
    .keys(move || store::message_ids(session_id))
    .alignment(ListAlignment::Bottom)
    .scrollbar(false)
    .content_padding_x(px(sizes::CARD_PADDING))
    .content_padding_y(px(24.0))
    .estimated_item_size(px(96.0))
    .row_gap(px(16.0))
    .empty(empty::empty(
        "还没有消息",
        "在下面输入一句话，Aedt 会转给 DeepSeek 回答",
    ))
    .flex_1()
}

/// A cell, made once and given other messages as it is reused.
fn message_row(session_id: u32, at: Position) -> impl IntoElement {
    let message = computed(move || store::message_at(session_id, at.index()));
    let from_user =
        move || message.with(|message| message.as_ref().is_some_and(|m| m.author == Author::User));

    view().w_full().child(
        when(from_user, move || user_message(message))
            .otherwise(move || assistant_message(message)),
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
        if !body.is_empty() {
            drop(set_clipboard(body));
        }
    };

    button::ghost_icon("⧉", "复制消息").on_press(copy)
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
            .text_color(move || theme().muted_foreground)
        },
    )
}

fn assistant_message(message: Memo<Option<Message>>) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
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
                .line_height(LineHeight::Multiple(MESSAGE_LINE))
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
