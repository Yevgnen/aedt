//! The app's multi-line composer field.

use nsh_base::{Input, input, theme};
use nsh_core::prelude::*;

use crate::ui::sizes;

/// A controlled composer bound to `value`; the owner keeps the text. It has
/// no border of its own: the floating card around it shows the focus.
pub(crate) fn composer(value: Signal<String>) -> Input {
    input()
        .value(move || value.get())
        .on_change(move |text: &str| value.set(text.to_owned()))
        .caret_color(move || color(theme().primary))
        .multiline(true)
        .scrollbar(false)
        .min_h(px(sizes::COMPOSER_MIN))
        .text_sm()
        .text_color(move || theme().foreground)
}
