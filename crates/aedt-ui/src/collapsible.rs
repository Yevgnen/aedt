//! A section of settings: a header that opens what it holds.

use nsh_base::{CollapsibleElement, collapsible, theme};
use nsh_core::element::AnyElement;
use nsh_core::prelude::*;

use crate::{icon, motion, state};

/// A section: `title` and a `mark` beside it always show; pressing the header
/// opens `content` and pressing it again closes it. It starts closed.
pub fn section(
    title: impl IntoReactive<SharedString>,
    mark: impl IntoElement,
    content: impl IntoElement,
) -> CollapsibleElement {
    row(title, Some(mark.into_element()), None, content)
}

/// A section whose header starts with `leading`, before the part that opens it:
/// a switch the section's subject takes at once, which the header's own press
/// would open the section instead of.
pub fn section_with_leading(
    title: impl IntoReactive<SharedString>,
    leading: impl IntoElement,
    content: impl IntoElement,
) -> CollapsibleElement {
    row(title, None, Some(leading.into_element()), content)
}

/// A section's header row: what leads it, then the part that opens it.
/// Neither takes the other's presses.
fn row(
    title: impl IntoReactive<SharedString>,
    mark: Option<AnyElement>,
    leading: Option<AnyElement>,
    content: impl IntoElement,
) -> CollapsibleElement {
    let open = signal(false);
    let mut handle = view()
        .flex_1()
        .min_w(px(0.0))
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
        .duration(motion::row_duration())
        .child(
            text(title)
                .flex_1()
                .text_sm()
                .font_weight(FontWeight::NORMAL)
                .text_color(move || theme().foreground),
        );
    if let Some(mark) = mark {
        handle = handle.child(mark);
    }
    handle = handle.child(
        // Points down while closed, up while open.
        view()
            .flex_none()
            .text_color(move || theme().muted_foreground)
            .style_when(move || open.get(), |s| s.rotate(deg(180.0)))
            .child(icon::chevron_down(14.0)),
    );
    let mut header = view().w_full().flex_row().items_center().gap_2();
    if let Some(leading) = leading {
        header = header.child(leading);
    }
    header = header.child(handle);

    collapsible()
        .open(open)
        .gap_4()
        .content(content)
        .child(header)
}
