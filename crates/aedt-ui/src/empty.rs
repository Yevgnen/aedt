//! What a region shows when it has nothing yet.

use nsh_base::theme;
use nsh_core::prelude::*;

/// One quiet line naming where to begin.
pub fn empty(label: impl IntoReactive<SharedString>) -> impl IntoElement {
    view().flex_1().items_center().justify_center().p_6().child(
        text(label)
            .text_sm()
            .text_color(move || theme().muted_foreground),
    )
}
