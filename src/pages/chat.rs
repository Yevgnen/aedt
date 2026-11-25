//! The conversation: its header, its message timeline and its composer.

use nsh_base::{text_view, theme};
use nsh_core::prelude::*;

use crate::{
    store::{self, Author, Message},
    ui::{avatar, badge, button, empty, input, motion, sizes},
};

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
        .child(timeline(session_id))
        .child(composer(session_id, draft))
}

fn header(session_id: u32) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_6()
        .py_4()
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
        .child(badge::badge("mock-model-1"))
}

fn timeline(session_id: u32) -> impl IntoElement {
    virtual_list(
        move || store::message_count(session_id),
        move |at| message_row(session_id, at),
    )
    .keys(move || store::message_ids(session_id))
    .alignment(ListAlignment::Bottom)
    .estimated_item_size(px(96.0))
    .content_padding(px(24.0))
    .row_gap(px(16.0))
    .empty(empty::empty(
        "还没有消息",
        "在下面输入一句话，Aedt 会用 Mock 数据回复",
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
    view().w_full().flex_row().justify_end().child(
        view()
            .max_w(px(sizes::MESSAGE_MAX))
            .rounded(move || theme().radius)
            .bg(move || theme().primary)
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
                .selectable(true)
                .text_color(move || theme().primary_foreground),
            ),
    )
}

fn assistant_message(message: Memo<Option<Message>>) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .items_start()
        .gap_3()
        .child(avatar::avatar("A"))
        .child(
            view()
                .flex_1()
                .min_w(px(0.0))
                .gap_1()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            text("Aedt")
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(when(
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
                        )),
                )
                .child(
                    text_view()
                        .markdown(move || {
                            message.with(|message| {
                                message.as_ref().map(|message| message.body.clone())
                            })
                        })
                        .selectable(true)
                        .gap_2()
                        .text_size(px(14.0))
                        .text_color(move || theme().foreground),
                ),
        )
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
        .w_full()
        .max_w(px(sizes::COMPOSER_MAX))
        .self_center()
        .px_6()
        .pt_4()
        .pb_3()
        .gap_2()
        .child(composer_card(draft, can_send, submit))
        .child(
            view()
                .h_5()
                .pl_3()
                .justify_center()
                .child(when(can_send, move || echo_status(draft))),
        )
}

/// The floating composer: a rounded card whose input sits on top and whose
/// round send button sits in the toolbar row below. The card carries the focus
/// edge, since the input itself is borderless.
fn composer_card(
    draft: Signal<String>,
    can_send: impl Fn() -> bool + Copy + 'static,
    submit: impl Fn() + Copy + 'static,
) -> impl IntoElement {
    view()
        .p_3()
        .gap_2()
        .rounded_3xl()
        .border_1()
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .shadow_sm()
        .focus_within(|s| s.border_color(move || theme().ring))
        .transition_colors()
        .duration(motion::control_duration())
        .ease_out()
        .child(input::composer(draft).label("消息"))
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_end()
                .child(button::icon("↑", "发送", can_send).on_press(submit)),
        )
}

/// The live readout under the card: what the mock model reports about the
/// draft while it changes, so the round trip is visible before sending.
fn echo_status(draft: Signal<String>) -> impl IntoElement {
    let chars = computed(move || draft.with(|draft| draft.chars().count()));
    let tokens = computed(move || chars.get() / 3 + 1);
    let enter = motion().entering(motion::fade_rise());

    view()
        .flex_row()
        .items_center()
        .gap_2()
        .bind_motion(&enter)
        .child(
            text(move || {
                format!(
                    "回显 {} 字 · {} tokens · {} ms",
                    chars.get(),
                    tokens.get(),
                    40 + chars.get() % 60
                )
            })
            .text_xs()
            .text_color(move || theme().muted_foreground),
        )
}
