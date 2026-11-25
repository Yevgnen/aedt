//! The session list: a persistent sidebar beside the conversation.

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::{
    store::{self, Session},
    ui::{
        avatar,
        button::{self, Variant},
        list_item, separator, sizes,
    },
};

pub(crate) fn sidebar() -> View {
    view()
        .w(px(sizes::SIDEBAR))
        .h_full()
        .bg(move || theme().sidebar)
        .border_r_1()
        .border_r_color(move || theme().sidebar_border)
        .child(brand())
        .child(
            view().px_3().pb_3().child(
                button::action(Variant::Secondary, "＋  新建会话")
                    .w_full()
                    .on_press(store::create_session),
            ),
        )
        .child(view().px_3().pb_2().child(separator::horizontal()))
        .child(sessions())
}

fn brand() -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .pt_4()
        .pb_3()
        .child(avatar::avatar("A"))
        .child(
            view()
                .flex_1()
                .min_w(px(0.0))
                .gap_1()
                .child(
                    text("Aedt")
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().sidebar_foreground),
                )
                .child(
                    text("Agent IDE")
                        .text_xs()
                        .text_color(move || theme().muted_foreground),
                ),
        )
}

fn sessions() -> impl IntoElement {
    virtual_list(store::session_count, session_row)
        .keys(store::session_ids)
        .estimated_item_size(px(sizes::LIST_ROW))
        .content_padding(px(8.0))
        .row_gap(px(2.0))
        .flex_1()
}

/// A cell, made once and given other sessions as it is reused: it keeps no
/// state, and every binding reads the session where that cell is now.
fn session_row(at: Position) -> impl IntoElement {
    let session = computed(move || store::session_at(at.index()));
    let id = move || session.with(|session| session.as_ref().map(|session| session.id));
    let selected = move || id() == Some(store::selected_session_id());

    list_item::session(
        move || {
            session.with(|session| {
                session
                    .as_ref()
                    .map(Session::display_title)
                    .unwrap_or_else(|| "新会话".into())
            })
        },
        move || {
            session.with(|session| {
                session
                    .as_ref()
                    .map(|session| format!("{} 条消息", session.messages.len()))
                    .unwrap_or_default()
            })
        },
        selected,
    )
    .accessibility_label(move || {
        Some(session.with(|session| {
            session
                .as_ref()
                .map(Session::display_title)
                .unwrap_or_else(|| "新会话".into())
        }))
    })
    .on_press(move || {
        if let Some(id) = id() {
            store::select_session(id);
        }
    })
}
