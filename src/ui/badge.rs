//! A short state or classification label.

use nsh_base::theme;
use nsh_core::prelude::*;

/// A neutral pill: for a state worth scanning, not for every label.
pub(crate) fn badge(label: impl IntoReactive<SharedString>) -> impl IntoElement {
    view()
        .rounded_full()
        .bg(move || theme().muted)
        .px_2()
        .py_1()
        .child(
            text(label)
                .text_xs()
                .text_color(move || theme().muted_foreground),
        )
}
