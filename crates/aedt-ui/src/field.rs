//! The app's form field: one visible label over one single-line input.

use nsh_base::{Input, input, theme};
use nsh_core::prelude::*;

use crate::{motion, sizes};

/// What a field does with what is typed.
pub enum Mask {
    /// The text shows.
    Plain,
    /// The text is hidden, as a key's is.
    Secret,
}

/// A field's label: one quiet line over the control it names.
pub fn label(name: impl IntoReactive<SharedString>) -> impl IntoElement {
    text(name)
        .text_xs()
        .text_color(move || theme().muted_foreground)
}

/// One field of a form: its label, then its input. The app owns the value:
/// `value` is what it holds, `on_change` hears the new text, and `on_submit`
/// hears return. The edge takes the focus color while the field has the
/// keyboard, so the field says where the keys are.
pub fn field(
    name: &'static str,
    value: impl Fn() -> String + Copy + 'static,
    on_change: impl Fn(&str) + Copy + 'static,
    on_submit: impl Fn() + Copy + 'static,
    mask: Mask,
) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        .child(label(name))
        .child(control(value, on_change, on_submit, mask))
}

/// The same field with `trailing` at the end of its edge, where a reset of the
/// value sits: the input keeps its own box, and its text stops short of the
/// control.
pub fn field_with_trailing(
    name: &'static str,
    value: impl Fn() -> String + Copy + 'static,
    on_change: impl Fn(&str) + Copy + 'static,
    on_submit: impl Fn() + Copy + 'static,
    mask: Mask,
    trailing: impl IntoElement,
) -> impl IntoElement {
    view().w_full().gap_1().child(label(name)).child(
        view()
            .relative()
            .w_full()
            .child(control(value, on_change, on_submit, mask).pr(px(sizes::CONTROL)))
            .child(
                view()
                    .absolute()
                    .right(px(4.0))
                    .top(px(0.0))
                    .bottom(px(0.0))
                    .items_center()
                    .justify_center()
                    .child(trailing),
            ),
    )
}

/// One field of a form whose value takes several lines, such as a list of
/// arguments: its label, then its input, `rows` lines tall at least.
pub fn multiline(
    name: &'static str,
    rows: u32,
    value: impl Fn() -> String + Copy + 'static,
    on_change: impl Fn(&str) + Copy + 'static,
) -> impl IntoElement {
    view().w_full().gap_1().child(label(name)).child(
        control(value, on_change, || {}, Mask::Plain)
            .multiline(true)
            .min_h(px(sizes::CONTROL * rows as f32))
            .py_2(),
    )
}

/// A field's input, in the app's field look.
fn control(
    value: impl Fn() -> String + Copy + 'static,
    on_change: impl Fn(&str) + Copy + 'static,
    on_submit: impl Fn() + Copy + 'static,
    mask: Mask,
) -> Input {
    input()
        .value(value)
        .on_change(on_change)
        .on_submit(move |_: &str| on_submit())
        .masked(matches!(mask, Mask::Secret))
        .auto_correct(false)
        .min_h(px(sizes::CONTROL))
        .px_2()
        .rounded(move || theme().radius)
        .border_1()
        .border_color(move || theme().border)
        .bg(move || theme().background)
        .text_sm()
        .text_color(move || theme().foreground)
        .focus(|s| s.border_color(move || theme().ring))
        .transition_colors()
        .duration(motion::row_duration())
}
