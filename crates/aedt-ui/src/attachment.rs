//! A file the reader attached to their next message: its name, and a press
//! takes it out of the message again.

use nsh_base::{ButtonElement, button, theme};
use nsh_core::{element::StyleRefinement, prelude::*};

use crate::{icon, motion, sizes, state};

/// A picked file's chip, called `name`. The whole chip is the target, so the
/// cross at its end reads as a hint rather than as a control of its own.
pub fn chip(name: SharedString) -> ButtonElement {
    let label = format!("Remove {name}");
    let reduce = reduce_motion();
    let press = move |s: StyleRefinement| {
        let s = s.bg(move || theme().accent);
        if reduce { s } else { s.scale(0.97) }
    };

    button()
        .accessibility_label(label)
        .h(px(sizes::CHIP))
        .max_w(px(sizes::CHIP_MAX_W))
        .flex_row()
        .items_center()
        .gap_1()
        .pl_2()
        .pr_1()
        .rounded_full()
        .bg(move || theme().muted)
        .hover(|s| s.bg(move || theme().accent))
        .active(press)
        .focus_visible(state::focus_ring)
        .hit_slop(EdgeInsets::all(px(2.0)))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(
            text(name)
                .min_w(px(0.0))
                .truncate()
                .text_xs()
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .flex_none()
                .text_color(move || theme().muted_foreground)
                .child(icon::x(12.0)),
        )
}
