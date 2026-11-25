//! The app's buttons: one press feedback, one focus ring and one disabled
//! look for every screen.

use nsh_base::{ButtonElement, button, theme};
use nsh_core::{element::StyleRefinement, prelude::*};

use crate::ui::{motion, sizes};

/// An ordinary labeled action. The caller adds `.on_press` and its layout.
pub(crate) fn secondary(label: impl IntoReactive<SharedString>) -> ButtonElement {
    let reduce = reduce_motion();
    let press = move |s: StyleRefinement| {
        let s = s.bg(move || theme().muted);
        if reduce { s } else { s.scale(0.97) }
    };

    button()
        .min_h(px(sizes::CONTROL))
        .px_3()
        .items_center()
        .justify_center()
        .gap_2()
        .rounded(move || theme().radius)
        .bg(move || theme().secondary)
        .hover(|s| s.bg(move || theme().accent))
        .active(press)
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .disabled_style(|s| s.opacity(0.4))
        .transition_all()
        .duration(motion::control_duration())
        .ease_out()
        .child(
            text(label)
                .text_sm()
                .text_color(move || theme().secondary_foreground),
        )
}

/// A circular icon button for one well-known action, enabled while `enabled`
/// holds.
pub(crate) fn icon(
    glyph: &'static str,
    accessibility_label: &'static str,
    enabled: impl Fn() -> bool + Copy + 'static,
) -> ButtonElement {
    let reduce = reduce_motion();
    let press = move |s: StyleRefinement| {
        let s = s.opacity(0.86);
        if reduce { s } else { s.scale(0.95) }
    };

    button()
        .accessibility_label(accessibility_label)
        .disabled(move || !enabled())
        .size(px(sizes::SEND))
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(move || {
            if enabled() {
                theme().primary
            } else {
                theme().muted
            }
        })
        .hover(|s| s.bg(move || theme().primary.opacity(0.9)))
        .active(press)
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .hit_slop(EdgeInsets::all(px(6.0)))
        .transition_all()
        .duration(motion::control_duration())
        .ease_out()
        .child(
            text(glyph)
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || {
                    if enabled() {
                        theme().primary_foreground
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
}
