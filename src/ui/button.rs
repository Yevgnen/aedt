//! The app's buttons: one press feedback, one focus ring and one disabled
//! look for every screen.

use nsh_base::{ButtonElement, button, theme};
use nsh_core::{element::StyleRefinement, prelude::*};

use crate::ui::{icon, motion, sizes};

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

/// A quiet action: a mark and its label on one line, no fill at rest, a wash on
/// hover. The caller adds `.on_press`, its ink and its place in the layout.
pub(crate) fn quiet(
    mark: impl IntoElement,
    label: impl IntoReactive<SharedString>,
) -> ButtonElement {
    button()
        .flex_row()
        .items_center()
        .gap_2()
        .px_1()
        .py(px(2.0))
        .rounded(move || theme().radius)
        .hover(|s| s.bg(move || theme().muted))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .transition_colors()
        .duration(motion::control_duration())
        .ease_out()
        .child(mark)
        .child(text(label).text_xs())
}

/// A control whose children draw all of its appearance, for a row whose hover
/// is its child's own: the shared focus ring and nothing else.
pub(crate) fn bare() -> ButtonElement {
    button()
        .rounded(move || theme().radius)
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
}

/// A button that adds to the list it stands before: a dashed edge around a
/// centered label, in the app's quiet ink.
pub(crate) fn add(label: impl IntoReactive<SharedString>) -> ButtonElement {
    button()
        .w_full()
        .flex_row()
        .items_center()
        .justify_center()
        .gap_2()
        .py_3()
        .rounded(move || theme().radius)
        .border_1()
        .border_dashed()
        .border_color(move || theme().border)
        .text_color(move || theme().muted_foreground)
        .hover(|s| s.bg(move || theme().muted))
        .active(|s| s.bg(move || theme().accent))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .disabled_style(|s| s.opacity(0.4))
        .transition_colors()
        .duration(motion::control_duration())
        .ease_out()
        .child(icon::plus(14.0))
        .child(text(label).text_sm())
}

/// A command that is only its label: no edge, no fill, a wash on hover — the
/// text form of [`ghost_icon`].
pub(crate) fn ghost(label: impl IntoReactive<SharedString>) -> ButtonElement {
    button()
        .min_h(px(sizes::CONTROL))
        .px_3()
        .items_center()
        .justify_center()
        .rounded(move || theme().radius)
        .text_color(move || theme().foreground)
        .hover(|s| s.bg(move || theme().muted))
        .active(|s| s.bg(move || theme().accent))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .disabled_style(|s| s.opacity(0.4))
        .transition_colors()
        .duration(motion::control_duration())
        .ease_out()
        .child(text(label).text_sm())
}

/// A quiet icon button for a row's trailing action: no fill at rest, a wash on
/// hover, and the same press and focus feedback as the other buttons.
pub(crate) fn ghost_icon(
    icon: impl IntoElement,
    accessibility_label: &'static str,
) -> ButtonElement {
    let reduce = reduce_motion();
    let press = move |s: StyleRefinement| {
        let s = s.bg(move || theme().muted);
        if reduce { s } else { s.scale(0.94) }
    };

    button()
        .accessibility_label(accessibility_label)
        .size(px(sizes::META_ACTION))
        .items_center()
        .justify_center()
        .rounded(move || theme().radius)
        .hover(|s| s.bg(move || theme().muted))
        .active(press)
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .disabled_style(|s| s.opacity(0.4))
        .hit_slop(EdgeInsets::all(px(4.0)))
        .transition_all()
        .duration(motion::control_duration())
        .ease_out()
        .child(icon)
}
