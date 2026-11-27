//! What a message carries: a file as its name, an image as itself.

use std::path::Path;

use nsh_base::{ButtonElement, button, theme};
use nsh_core::{element::StyleRefinement, prelude::*};

use crate::{icon, motion, sizes, state};

/// Whether `path` names one of the formats a message carries as an image: the
/// extensions of the four the agent reads from the file's first bytes.
pub fn is_image(path: &Path) -> bool {
    const IMAGES: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            IMAGES
                .iter()
                .any(|known| extension.eq_ignore_ascii_case(known))
        })
}

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

/// An image the next message carries, shown as itself: a rounded tile of
/// `path`, with a cross over its corner that takes it out.
pub fn thumbnail(
    path: SharedString,
    name: SharedString,
    remove: impl Fn() + 'static,
) -> impl IntoElement {
    view()
        .relative()
        .size(px(sizes::THUMB))
        .flex_none()
        .rounded(move || theme().radius)
        .overflow_hidden()
        .bg(move || theme().muted)
        .child(
            image(path)
                .size(px(sizes::THUMB))
                .object_fit(ResizeMode::Cover),
        )
        .child(
            view()
                .absolute()
                .top(px(3.0))
                .right(px(3.0))
                .child(remove_button(name, remove)),
        )
}

/// A picture a sent message carried, shown as a message of its own: the picture
/// whole in a rounded tile, with nothing behind it, so the page it showed reads
/// as it was sent.
pub fn picture(path: SharedString) -> impl IntoElement {
    view()
        .w(px(sizes::PICTURE_W))
        .h(px(sizes::PICTURE_H))
        .flex_none()
        .rounded(move || theme().radius)
        .overflow_hidden()
        .child(
            image(path)
                .w(px(sizes::PICTURE_W))
                .h(px(sizes::PICTURE_H))
                .object_fit(ResizeMode::Contain),
        )
}

/// The cross over an attached image's corner: a small round target on the
/// card's fill, so it reads over whatever the picture beneath it is.
fn remove_button(name: SharedString, remove: impl Fn() + 'static) -> ButtonElement {
    let label = format!("Remove {name}");
    let reduce = reduce_motion();
    let press = move |s: StyleRefinement| {
        let s = s.opacity(0.85);
        if reduce { s } else { s.scale(0.94) }
    };

    button()
        .accessibility_label(label)
        .size(px(sizes::META_ACTION))
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(move || theme().card.opacity(0.9))
        .hover(|s| s.bg(move || theme().card))
        .active(press)
        .focus_visible(state::focus_ring)
        .hit_slop(EdgeInsets::all(px(4.0)))
        .transition_all()
        .duration(motion::row_duration())
        .ease_out()
        .text_color(move || theme().foreground)
        .on_press(remove)
        .child(icon::x(12.0))
}
