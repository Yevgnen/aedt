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

/// A section whose header carries `trailing` before its chevron: a switch the
/// section's subject takes at once, which the header's own press would open the
/// section instead of.
pub fn section_with_trailing(
    title: impl IntoReactive<SharedString>,
    trailing: impl IntoElement,
    content: impl IntoElement,
) -> CollapsibleElement {
    row(title, None, Some(trailing.into_element()), content)
}

/// A section's header row: the part that opens it, then what stands after it,
/// then the chevron that ends it. What stands after the title keeps its own
/// presses; the chevron opens the section as the title does.
fn row(
    title: impl IntoReactive<SharedString>,
    mark: Option<AnyElement>,
    trailing: Option<AnyElement>,
    content: impl IntoElement,
) -> CollapsibleElement {
    let open = signal(false);
    let flip = move || open.update(|open| *open = !*open);
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
        .on_press(flip)
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
    let mut header = view()
        .w_full()
        .flex_row()
        .items_center()
        .gap_2()
        .child(handle);
    if let Some(trailing) = trailing {
        header = header.child(trailing);
    }
    header = header.child(
        // Points down while closed, up while open. The title says what it does,
        // so screen readers skip it.
        view()
            .flex_none()
            .text_color(move || theme().muted_foreground)
            .hit_slop(EdgeInsets::all(px(6.0)))
            .accessibility_elements_hidden(true)
            .on_press(flip)
            .style_when(move || open.get(), |s| s.rotate(deg(180.0)))
            .child(icon::chevron_down(14.0)),
    );

    collapsible()
        .open(open)
        .gap_4()
        .content(content)
        .child(header)
}
