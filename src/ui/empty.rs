//! What a region shows when it has nothing yet.

use nsh_base::theme;
use nsh_core::prelude::*;

/// A title and one line that names the next action.
pub(crate) fn empty(
    title: impl IntoReactive<SharedString>,
    detail: impl IntoReactive<SharedString>,
) -> impl IntoElement {
    view()
        .flex_1()
        .items_center()
        .justify_center()
        .gap_2()
        .p_6()
        .child(
            text(title)
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().foreground),
        )
        .child(
            text(detail)
                .text_xs()
                .number_of_lines(2)
                .text_color(move || theme().muted_foreground),
        )
}
