//! The app's buttons: one press feedback, one focus ring and one disabled
//! look for every screen.

use nsh_base::{ButtonElement, button, theme};
use nsh_core::{element::StyleRefinement, prelude::*};

use crate::{icon, motion, sizes, state, tokens};

/// An ordinary labeled action. The caller adds `.on_press` and its layout.
pub fn secondary(label: impl IntoReactive<SharedString>) -> ButtonElement {
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
        .focus_visible(state::focus_ring)
        .disabled_style(|s| s.opacity(0.4))
        .transition_all()
        .duration(motion::row_duration())
        .ease_out()
        .child(
            text(label)
                .text_sm()
                .text_color(move || theme().secondary_foreground),
        )
}

/// A circular icon button for one well-known action, enabled while `enabled`
/// holds.
pub fn icon(
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
        .focus_visible(state::focus_ring)
        .hit_slop(EdgeInsets::all(px(6.0)))
        .transition_all()
        .duration(motion::row_duration())
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

/// A round icon button for one action beside the composer's send: the same
/// size and press as [`icon`], in the quiet fill the card already uses, so a
/// secondary action does not compete with the send.
pub fn quiet_icon(mark: impl IntoElement, accessibility_label: &'static str) -> ButtonElement {
    let reduce = reduce_motion();
    let press = move |s: StyleRefinement| {
        let s = s.bg(move || theme().accent);
        if reduce { s } else { s.scale(0.95) }
    };

    button()
        .accessibility_label(accessibility_label)
        .size(px(sizes::SEND))
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(move || theme().muted)
        .hover(|s| s.bg(move || theme().accent))
        .active(press)
        .focus_visible(state::focus_ring)
        .hit_slop(EdgeInsets::all(px(6.0)))
        .transition_all()
        .duration(motion::row_duration())
        .ease_out()
        .child(view().text_color(move || theme().foreground).child(mark))
}

/// A round icon button that floats over the content it acts on: the popup
/// surface and the floating shadow, so it reads as laid on the conversation
/// rather than in it. The caller places it.
pub fn floating_icon(mark: impl IntoElement, accessibility_label: &'static str) -> ButtonElement {
    let reduce = reduce_motion();
    let press = move |s: StyleRefinement| {
        let s = s.bg(move || theme().accent);
        if reduce { s } else { s.scale(0.95) }
    };

    button()
        .accessibility_label(accessibility_label)
        .size(px(sizes::CONTROL))
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(move || theme().popover)
        .border_1()
        .border_color(move || theme().border)
        .shadow(tokens::floating_shadow())
        .hover(|s| s.bg(move || theme().accent))
        .active(press)
        .focus_visible(state::focus_ring)
        .hit_slop(EdgeInsets::all(px(4.0)))
        .transition_all()
        .duration(motion::row_duration())
        .ease_out()
        .child(view().text_color(move || theme().foreground).child(mark))
}

/// A control whose children draw all of its appearance, for a row whose hover
/// is its child's own: the shared focus ring and nothing else.
pub fn bare() -> ButtonElement {
    button()
        .rounded(move || theme().radius)
        .focus_visible(state::focus_ring)
}

/// A button that adds to the list it stands before: a dashed edge around a
/// centered label, in the app's quiet ink.
pub fn add(label: impl IntoReactive<SharedString>) -> ButtonElement {
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
        .focus_visible(state::focus_ring)
        .disabled_style(|s| s.opacity(0.4))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(icon::plus(14.0))
        .child(text(label).text_sm())
}

/// A command that is only its label: no edge, no fill, a wash on hover — the
/// text form of [`ghost_icon`].
pub fn ghost(label: impl IntoReactive<SharedString>) -> ButtonElement {
    button()
        .min_h(px(sizes::CONTROL))
        .px_3()
        .items_center()
        .justify_center()
        .rounded(move || theme().radius)
        .text_color(move || theme().foreground)
        .hover(|s| s.bg(move || theme().muted))
        .active(|s| s.bg(move || theme().accent))
        .focus_visible(state::focus_ring)
        .disabled_style(|s| s.opacity(0.4))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(text(label).text_sm())
}

/// A quiet icon button for a row's trailing action: no fill at rest, a wash on
/// hover, and the same press and focus feedback as the other buttons.
pub fn ghost_icon(icon: impl IntoElement, accessibility_label: &'static str) -> ButtonElement {
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
        .rounded(px(sizes::ICON_RADIUS))
        .hover(|s| s.bg(move || theme().muted))
        .active(press)
        .focus_visible(state::focus_ring)
        .disabled_style(|s| s.opacity(0.4))
        .hit_slop(EdgeInsets::all(px(4.0)))
        .transition_all()
        .duration(motion::row_duration())
        .ease_out()
        .child(icon)
}
