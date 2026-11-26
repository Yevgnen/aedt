//! A short state or classification label.

use nsh_base::theme;
use nsh_core::prelude::*;

/// A neutral pill: for a state worth scanning, not for every label.
pub fn badge(label: impl IntoReactive<SharedString>) -> impl IntoElement {
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

/// A pill that stands beside text on the same line, such as the value a
/// control shows: its label takes that text's size and line, so both sit on
/// one baseline, and its padding is the smaller part of a row's.
pub fn tag(label: impl IntoReactive<SharedString>) -> impl IntoElement {
    view()
        .rounded_full()
        .bg(move || theme().muted)
        .px_2()
        .py_0p5()
        .child(
            text(label)
                .text_size(px(13.0))
                .text_color(move || theme().muted_foreground),
        )
}
