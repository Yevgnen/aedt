//! The rail beside the conversation: one tick per question, and the card a tick
//! opens.

use aedt_store as store;
use aedt_ui::{button, motion, sizes, surface, tokens};
use nsh_base::theme;
use nsh_core::prelude::*;

use super::body_of;

/// A tick's bar height, the padding that fills its pitch, and its length at
/// rest and lit: adjacent ticks' hit areas touch, so sliding along the column
/// never leaves it between them.
const TICK_HEIGHT: f32 = 2.0;
const TICK_PAD: f32 = 4.0;
const TICK_LEN: f32 = 10.0;
const TICK_LEN_LIT: f32 = 18.0;
pub(super) fn rail(
    session_id: u32,
    timeline: ElementRef,
    current: Signal<Option<usize>>,
) -> impl IntoElement {
    // Which question's card shows, as (its row among the ticks, its message
    // index): one card for the whole column, so moving from tick to tick swaps
    // its content instead of dropping it for a frame.
    let hovered = signal(None::<(usize, usize)>);
    let height = signal(0.0f32);

    view()
        .w(px(sizes::RAIL))
        .pr(px(sizes::RAIL_INSET))
        .flex_none()
        .flex_col()
        .items_center()
        .justify_center()
        .on_mouse_leave(move |_: &MouseEvent| hovered.set(None))
        .on_layout(move |bounds: &Bounds<Points>| height.set(bounds.size.height.0))
        .child(each(
            move || store::user_message_indices(session_id),
            |index| *index,
            move |index, row| tick(index, row, timeline, current, hovered),
        ))
        .child(when(
            move || hovered.get().is_some(),
            move || message_preview(session_id, hovered, height),
        ))
}

/// One question's tick.
fn tick(
    at: ReadSignal<usize>,
    row: ReadSignal<usize>,
    timeline: ElementRef,
    current: Signal<Option<usize>>,
    hovered: Signal<Option<(usize, usize)>>,
) -> impl IntoElement {
    // Lit: the tick under the pointer, and the reader's place in the list.
    // A memo of its own, so the pointer moving over another tick leaves every
    // other tick's style alone: the signal changes, this tick's answer does
    // not, and nothing under it runs.
    let lit = computed(move || {
        hovered.get().is_some_and(|(_, index)| index == at.get()) || current.get() == Some(at.get())
    });

    let jump = move || {
        timeline.scroll_to_index(at.get() as u32, percentage(0.0), Points::ZERO, true);
    };

    button::bare()
        .accessibility_label(move || {
            Some(SharedString::from(format!(
                "Jump to question {}",
                row.get() + 1
            )))
        })
        .w_full()
        .items_end()
        // The hit area fills the pitch, so sliding along the column never
        // leaves it between two ticks.
        .py(px(TICK_PAD))
        .on_mouse_enter(move |_: &MouseEvent| hovered.set(Some((row.get(), at.get()))))
        .cursor_pointer()
        .on_press(jump)
        .child(
            // Every tick rests light and short; the lit ones — under the
            // pointer, or where the reader is — are dark and long. The length
            // is a scale over a fixed bar: a transition moves colors and
            // transforms, not widths, so a bar that grew would jump while its
            // color eased.
            view()
                .h(px(TICK_HEIGHT))
                .w(px(TICK_LEN_LIT))
                .rounded_full()
                .origin_right()
                .scale_x(move || {
                    if lit.get() {
                        1.0
                    } else {
                        TICK_LEN / TICK_LEN_LIT
                    }
                })
                .bg(move || {
                    if lit.get() {
                        theme().primary_foreground
                    } else {
                        theme().border
                    }
                })
                .transition_all()
                .duration(motion::row_duration())
                .ease_out(),
        )
}

/// The card beside the column: the question under the pointer, then what it was
/// answered. It keeps its place in the tree while the pointer moves from tick to
/// tick, and only its content and offset change.
fn message_preview(
    session_id: u32,
    hovered: Signal<Option<(usize, usize)>>,
    height: Signal<f32>,
) -> impl IntoElement {
    let question = computed(move || {
        hovered
            .get()
            .and_then(|(_, index)| store::message_at(session_id, index))
    });
    let reply = computed(move || {
        hovered
            .get()
            .and_then(|(_, index)| store::reply_at(session_id, index))
    });

    surface::card(surface::Surface::Preview)
        .absolute()
        // The card ends clear of the column and a little more: the lit tick is
        // the widest of them, reaching the rail's left edge, and would show
        // under the card's own edge.
        .right(px(sizes::RAIL + 6.0))
        .top(move || {
            let (row, _) = hovered.get().unwrap_or((0, 0));
            let rows = store::user_message_count(session_id) as f32;
            let pitch = TICK_HEIGHT + TICK_PAD * 2.0;
            let start = (height.get() - rows * pitch) / 2.0;
            px(start + row as f32 * pitch - 20.0)
        })
        .w(px(sizes::PREVIEW_W))
        .p_3()
        .gap_2()
        .child(
            text(body_of(question))
                .text_sm()
                .number_of_lines(3)
                .text_color(move || theme().foreground),
        )
        .child(when(
            move || reply.with(|reply| reply.is_some()),
            move || {
                // The answer sits one point smaller and a shade quieter than
                // the question it belongs to.
                text(body_of(reply))
                    .text_size(px(sizes::TEXT_COMPACT))
                    .number_of_lines(4)
                    .text_color(move || tokens::meta().opacity(0.55))
            },
        ))
}
