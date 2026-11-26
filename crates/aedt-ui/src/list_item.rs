//! The app's session rows and settings destinations: one navigation row.

use nsh_base::{ButtonElement, button, theme};
use nsh_core::prelude::*;

use crate::{motion, sizes};

/// A conversation's title: a point under a destination's, so the list of
/// conversations reads quieter than the pages' names.
const SESSION_TITLE: f32 = 13.0;

/// A settings destination's title.
const DESTINATION_TITLE: f32 = 14.0;

/// A navigation row: its title, selected by the caller, which adds `.on_press`.
pub fn nav(
    title: impl IntoReactive<SharedString>,
    selected: impl Fn() -> bool + Copy + 'static,
) -> ButtonElement {
    nav_row(None, title, selected, SESSION_TITLE)
}

/// A navigation row with a mark before its title, as a settings destination
/// has; the caller adds `.on_press` as it does for [`nav`].
pub fn nav_with_mark(
    mark: impl IntoElement,
    title: impl IntoReactive<SharedString>,
    selected: impl Fn() -> bool + Copy + 'static,
) -> ButtonElement {
    nav_row(
        Some(mark.into_element()),
        title,
        selected,
        DESTINATION_TITLE,
    )
}

fn nav_row(
    mark: Option<AnyElement>,
    title: impl IntoReactive<SharedString>,
    selected: impl Fn() -> bool + Copy + 'static,
    title_size: f32,
) -> ButtonElement {
    button()
        .selected(selected)
        .w_full()
        .min_h(px(sizes::LIST_ROW))
        .flex_row()
        .items_center()
        .gap_2()
        .px(px(sizes::LIST_ROW_INSET))
        .py_1()
        .rounded(move || theme().radius)
        // NSH ranks `hover:` above `selected:` (`rank::HOVER` 34 > `rank::SELECTED`
        // 32), so a selected row keeps the selected fill by taking that same
        // color while hovered.
        .hover(|s| {
            s.bg(move || {
                if selected() {
                    theme().accent
                } else {
                    theme().muted
                }
            })
        })
        .active(|s| s.bg(move || theme().accent))
        .selected_style(|s| s.bg(move || theme().accent))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .when_some(mark, |row, mark| row.child(mark))
        .child(
            text(title)
                .flex_1()
                .min_w(px(0.0))
                .text_size(px(title_size))
                .font_weight(FontWeight::NORMAL)
                .number_of_lines(1)
                .text_color(move || theme().foreground),
        )
}
