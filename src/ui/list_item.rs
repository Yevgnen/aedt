//! The app's session row in a navigation list.

use nsh_base::{ButtonElement, button, theme};
use nsh_core::prelude::*;

use crate::ui::{motion, sizes};

/// A session row: its title, selected by the caller.
pub(crate) fn session(
    title: impl IntoReactive<SharedString>,
    selected: impl Fn() -> bool + Copy + 'static,
) -> ButtonElement {
    button()
        .selected(selected)
        .w_full()
        .min_h(px(sizes::LIST_ROW))
        .flex_row()
        .items_center()
        .px(px(sizes::LIST_ROW_INSET))
        .py_1()
        .rounded(move || theme().radius)
        .hover(|s| s.bg(move || theme().muted))
        .active(|s| s.bg(move || theme().accent))
        .selected_style(|s| s.bg(move || theme().accent))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(
            text(title)
                .flex_1()
                .min_w(px(0.0))
                .text_sm()
                .font_weight(FontWeight::NORMAL)
                .number_of_lines(1)
                .text_color(move || {
                    if selected() {
                        theme().foreground
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
}
