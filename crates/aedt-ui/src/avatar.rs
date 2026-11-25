//! A round identity mark.

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::sizes;

/// A circle with the first letter of the name it stands for.
pub fn avatar(initial: &'static str) -> impl IntoElement {
    view()
        .size(px(sizes::AVATAR))
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(move || theme().secondary)
        .child(
            text(initial)
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().secondary_foreground),
        )
}
