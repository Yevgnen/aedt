//! The app's multi-line composer field.

use nsh_base::{Input, input, theme};
use nsh_core::{element::Selection, prelude::*};

use crate::sizes;

/// A controlled composer bound to `value`; the owner keeps the text and sends
/// it with `on_send`, and hears where the caret moves with `on_selection`.
/// Return sends and keeps the focus; Shift+Return and Control+Return break the
/// line instead. It has no border of its own: the floating card around it is
/// the frame.
pub fn composer(
    value: Signal<String>,
    on_send: impl Fn() + Copy + 'static,
    on_selection: impl Fn(Selection) + Copy + 'static,
) -> Input {
    input()
        .value(move || value.get())
        .on_change(move |text: &str| value.set(text.to_owned()))
        .on_selection_change(move |selection: &Selection| on_selection(*selection))
        .multiline(true)
        .submit_behavior(SubmitBehavior::Submit)
        .on_submit(move |_: &str| on_send())
        .scrollbar(false)
        .min_h(px(sizes::COMPOSER_MIN))
        .text_sm()
        .text_color(move || theme().foreground)
}
