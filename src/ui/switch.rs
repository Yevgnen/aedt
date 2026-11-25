//! The app's switch: a setting that takes effect at once.

use nsh_base::{SwitchElement, switch, theme};
use nsh_core::prelude::*;

use crate::ui::{motion, tokens};

/// A setting's on/off switch: the app's track and thumb. `label` is what
/// screen readers call it (the row beside it says it in words); `checked` is
/// the value the app owns, and `on_change` hears the one the press asks for.
pub(crate) fn toggle(
    label: impl Fn() -> SharedString + Copy + 'static,
    checked: impl Fn() -> bool + Copy + 'static,
    on_change: impl Fn(bool) + Copy + 'static,
) -> SwitchElement {
    switch()
        .checked(checked)
        .on_change(move |next: &bool| on_change(*next))
        .accessibility_label(label)
        .track_style(|s| {
            s.w(px(44.0))
                .p(px(2.0))
                .rounded_full()
                .bg(move || theme().muted)
                .transition_colors()
                .duration(motion::control_duration())
                .checked(|s| s.bg(tokens::checked))
        })
        .thumb_style(|s| {
            s.size(px(20.0))
                .rounded_full()
                .bg(move || theme().background)
                .shadow_sm()
                .transition_transform()
                .duration(motion::control_duration())
                .checked(|s| s.translate_x(px(20.0)))
        })
}
