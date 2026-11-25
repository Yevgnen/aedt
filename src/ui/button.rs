//! The app's button: one press feedback, one focus ring and one disabled
//! look for every screen.

use nsh_base::{ButtonElement, button, theme};
use nsh_core::{element::StyleRefinement, prelude::*};

use crate::ui::{motion, sizes};

/// What a button is for, which decides its emphasis.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Variant {
    /// The one commit of a decision area.
    Primary,
    /// An ordinary visible action.
    Secondary,
}

/// A labeled action. The caller adds `.on_press`, `.disabled` and its layout.
pub(crate) fn action(variant: Variant, label: impl IntoReactive<SharedString>) -> ButtonElement {
    let reduce = reduce_motion();
    let press = move |s: StyleRefinement| {
        let s = match variant {
            Variant::Primary => s.opacity(0.86),
            Variant::Secondary => s.bg(move || theme().muted),
        };
        if reduce { s } else { s.scale(0.97) }
    };

    let base = button()
        .min_h(px(sizes::CONTROL))
        .px_3()
        .items_center()
        .justify_center()
        .gap_2()
        .rounded(move || theme().radius)
        .transition_all()
        .duration(motion::control_duration())
        .ease_out()
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .disabled_style(|s| s.opacity(0.4))
        .active(press);

    match variant {
        Variant::Primary => base
            .bg(move || theme().primary)
            .hover(|s| s.bg(move || theme().primary.opacity(0.9)))
            .child(
                text(label)
                    .text_sm()
                    .text_color(move || theme().primary_foreground),
            ),
        Variant::Secondary => base
            .bg(move || theme().secondary)
            .hover(|s| s.bg(move || theme().accent))
            .child(
                text(label)
                    .text_sm()
                    .text_color(move || theme().secondary_foreground),
            ),
    }
}

/// A circular icon button for one well-known action.
pub(crate) fn icon(glyph: &'static str, accessibility_label: &'static str) -> ButtonElement {
    action(Variant::Primary, glyph)
        .accessibility_label(accessibility_label)
        .size(px(sizes::SEND))
        .px(px(0.0))
        .rounded_full()
}
