//! The app's form field: one visible label over one single-line input.

use nsh_base::{input, theme};
use nsh_core::prelude::*;

use crate::ui::{motion, sizes};

/// What a field does with what is typed.
pub(crate) enum Mask {
    /// The text shows.
    Plain,
    /// The text is hidden, as a key's is.
    Secret,
}

/// One field of a form: its label, then its input. The app owns the value:
/// `value` is what it holds, `on_change` hears the new text, and `on_submit`
/// hears return. The edge takes the focus color while the field has the
/// keyboard, so the field says where the keys are.
pub(crate) fn field(
    label: &'static str,
    value: impl Fn() -> String + Copy + 'static,
    on_change: impl Fn(&str) + Copy + 'static,
    on_submit: impl Fn() + Copy + 'static,
    mask: Mask,
) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        .child(
            text(label)
                .text_xs()
                .text_color(move || theme().muted_foreground),
        )
        .child(
            input()
                .value(value)
                .on_change(on_change)
                .on_submit(move |_: &str| on_submit())
                .masked(matches!(mask, Mask::Secret))
                .auto_correct(false)
                .min_h(px(sizes::CONTROL))
                .px_2()
                .rounded(move || theme().radius)
                .border_1()
                .border_color(move || theme().border)
                .bg(move || theme().background)
                .text_sm()
                .text_color(move || theme().foreground)
                .focus(|s| s.border_color(move || theme().ring))
                .transition_colors()
                .duration(motion::control_duration()),
        )
}
