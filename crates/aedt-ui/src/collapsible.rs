//! A section of settings: a header that opens what it holds.

use nsh_base::{CollapsibleElement, collapsible, theme};
use nsh_core::prelude::*;

use crate::{icon, motion, state};

/// A section: `title` and a `mark` beside it always show; pressing the header
/// opens `content` and pressing it again closes it. It starts closed.
pub fn section(
    title: impl IntoReactive<SharedString>,
    mark: impl IntoElement,
    content: impl IntoElement,
) -> CollapsibleElement {
    let open = signal(false);
    collapsible().open(open).gap_4().content(content).child(
        view()
            .w_full()
            .flex_row()
            .items_center()
            .gap_2()
            .rounded(move || theme().radius)
            .role(AccessibilityRole::Button)
            .accessibility_state(move || AccessibilityState {
                expanded: Some(open.get()),
                ..AccessibilityState::default()
            })
            .on_press(move || open.update(|open| *open = !*open))
            .focus_visible(state::focus_ring)
            .transition_colors()
            .duration(motion::control_duration())
            .child(
                text(title)
                    .flex_1()
                    .text_sm()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(move || theme().foreground),
            )
            .child(mark)
            .child(
                // Points down while closed, up while open.
                view()
                    .flex_none()
                    .text_color(move || theme().muted_foreground)
                    .style_when(move || open.get(), |s| s.rotate(deg(180.0)))
                    .child(icon::chevron_down(14.0)),
            ),
    )
}
