//! The app's multi-line composer field.

use nsh_base::{Input, input, theme};
use nsh_core::prelude::*;

use crate::ui::{motion, sizes};

/// A controlled composer bound to `value`; the owner keeps the text.
pub(crate) fn composer(value: Signal<String>) -> Input {
    input()
        .value(move || value.get())
        .on_change(move |text: &str| value.set(text.to_owned()))
        .caret_color(move || color(theme().primary))
        .multiline(true)
        .min_h(px(sizes::COMPOSER_MIN))
        .p_3()
        .text_sm()
        .text_color(move || theme().foreground)
        .rounded(move || theme().radius)
        .border_1()
        .border_color(move || theme().input)
        .focus(|s| s.border_color(move || theme().ring))
        .transition_colors()
        .duration(motion::control_duration())
        .ease_out()
}
