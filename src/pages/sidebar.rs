//! The session list: a persistent sidebar beside the conversation.

use aedt_store as store;
use aedt_ui::{avatar, button, icon, list_item, menu, separator, sizes};
use nsh_base::theme;
use nsh_core::{element::desktop::title_bar, prelude::*};

use crate::settings;

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
                button::secondary("+  New chat")
                    .w_full()
                    .on_press(store::create_session),
            ),
        )
        .child(view().px_3().pb_2().child(separator::horizontal()))
        .child(sessions())
        .child(footer())
}

/// The sidebar's foot, at the bottom edge it belongs to: the menu of the
/// app-wide commands.
fn footer() -> impl IntoElement {
    view().px_3().pb_3().child(menu::menu(
        menu::trigger(icon::more(14.0), "More"),
        Placement::Top,
        Align::Start,
        menu::item(icon::settings(14.0), "Settings").on_select(settings::open),
    ))
}

fn brand() -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        // The traffic lights own the whole title bar band on the left edge, so
        // the brand starts right below it: no extra padding above.
        .pt(move || Length::from(title_bar().top))
        .pb_4()
        .window_drag(true)
        .child(avatar::avatar("A"))
        .child(
            text("Aedt")
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().sidebar_foreground),
        )
}

fn sessions() -> impl IntoElement {
    virtual_list(store::session_count, session_row)
        .keys(store::session_ids)
        .scrollbar(false)
        .estimated_item_size(px(sizes::LIST_ROW))
        .content_padding(px(12.0))
        .row_gap(px(2.0))
        .flex_1()
}

/// A cell, made once and given other sessions as it is reused: it keeps no
/// state, and every binding reads the session where that cell is now.
fn session_row(at: Position) -> impl IntoElement {
    let session = computed(move || store::session_summary_at(at.index()));
    let id = move || session.with(|session| session.as_ref().map(|session| session.id));
    let title = move || {
        session.with(|session| {
            session
                .as_ref()
                .map(|session| session.title.clone())
                .unwrap_or_else(|| "New chat".into())
        })
    };
    let selected = move || id() == Some(store::selected_session_id());

    list_item::nav(title, selected)
        .accessibility_label(move || Some(title()))
        .on_press(move || {
            if let Some(id) = id() {
                store::select_session(id);
            }
        })
}
