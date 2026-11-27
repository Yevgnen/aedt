//! A call a reply made: its row, what it was given and answered while the row
//! is open, and the cards of the files it changed.

use aedt_store::{self as store, ToolRun};
use aedt_ui::{MESSAGE_LINE, button, code, diff, icon, markdown, motion, sizes, tokens};
use nsh_core::prelude::*;

use super::Folds;

/// The air a call's row keeps above and below it, which a thought that a call
/// follows ends with, so the two sit as close as two calls do.
pub(super) const ROW_INSET: f32 = 2.0;

pub(super) fn tool_run(tool: Memo<ToolRun>, folds: Folds) -> impl IntoElement {
    let id = move || tool.with(|tool| tool.id.clone());
    let shown = move || tool.with(|tool| folds.call_open(tool));
    let toggle = move || {
        let id = id();
        let open = !shown();

        folds.set_call(id, open);
    };

    view()
        .w_full()
        .gap_1()
        .child(
            button::bare()
                // A button cannot carry an expanded state of its own: its
                // label says which of the two it shows.
                .accessibility_label(move || {
                    let shown = shown();

                    Some(tool.with(|tool| {
                        SharedString::from(format!(
                            "{} result, {}",
                            name_of(tool.name.as_str()),
                            if shown { "shown" } else { "hidden" }
                        ))
                    }))
                })
                .w_full()
                .group()
                .flex_row()
                .items_center()
                .gap_2()
                .px_1()
                .py(px(ROW_INSET))
                .text_color(tokens::meta)
                .on_press(toggle)
                .child(
                    // The wrench and the name read in one ink, a shade under the
                    // row's quietest, which the arguments after them take still
                    // lighter.
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .text_color(|| tokens::meta().opacity(0.8))
                        .child(icon::wrench(14.0))
                        .child(
                            // The tool's name is prose about the call, not code
                            // in it: it reads at the size the thought's own name
                            // does, where the arguments and the file beside it
                            // are set in the code's.
                            text(move || {
                                tool.with(|tool| {
                                    SharedString::from(name_of(tool.name.as_str()).to_owned())
                                })
                            })
                            .flex_none()
                            .text_xs(),
                        ),
                )
                .child(view().flex_1().min_w(px(0.0)).child(summary(tool, shown)))
                .child(when(
                    move || tool.with(|tool| tool.running),
                    || text("running…").text_xs().text_color(tokens::meta),
                ))
                .child(when(
                    move || tool.with(|tool| tool.failed),
                    || text("failed").text_xs().text_color(tokens::caption),
                ))
                .child(
                    // The chevron is what says a press opens the call's output:
                    // it shows while the pointer is over the row, and for the
                    // keyboard's focus, leaving the reply quiet at rest.
                    view()
                        .opacity(0.0)
                        .group_hover(|s| s.opacity(1.0))
                        .group_focus_within(|s| s.opacity(1.0))
                        .transition_opacity()
                        .duration(motion::row_duration())
                        .ease_out()
                        .child(
                            when(shown, || icon::chevron_up(14.0))
                                .otherwise(|| icon::chevron_down(14.0)),
                        ),
                ),
        )
        // What the call was given and what it answered, while its row is open,
        // then the files an edit changed under them.
        .child(when(shown, move || data(tool)))
        .child(when(
            move || shown() && tool.with(|tool| !tool.changes.is_empty()),
            move || changes(tool),
        ))
}

/// What the row calls a tool: its name without the server that serves it. An
/// MCP tool is named `<server>__<tool>`, so what stands before the first `__`
/// goes, and a name without one is its own.
fn name_of(tool: &str) -> &str {
    tool.split_once("__").map_or(tool, |(_, name)| name)
}

/// What the row says between the name and the status: the file an edit changed,
/// or the arguments of a call that only runs something, while the row is shut.
/// An open row says nothing here: what the call was given is in the block under
/// it.
fn summary(tool: Memo<ToolRun>, shown: impl Fn() -> bool + Copy + 'static) -> Text {
    text(move || {
        tool.with(|tool| match tool.changes.first() {
            Some(change) => change.path.to_string(),
            None if shown() => String::new(),
            None => tool.arguments.to_string(),
        })
    })
    .w_full()
    .min_w(px(0.0))
    .number_of_lines(1)
    .font_family(code::MONO_FONT)
    .text_size(px(sizes::CODE_FONT))
    .text_color(tokens::caption)
}

/// Where a call's blocks start: under the word beside its wrench, past the
/// row's inset, the mark and the gap between them — as a thought's text starts
/// past its bulb.
const TOOL_TEXT_INSET: f32 = 26.0;

/// The lines of a block that stay in view before the rest scrolls.
const BLOCK_LINES: f32 = 16.0;

/// What a call was given and what it answered, while its row is open: the first
/// under `Input:`, the second under `Output:`.
fn data(tool: Memo<ToolRun>) -> impl IntoElement {
    view()
        .w_full()
        .pt_1()
        // A blank line of the code's own under the last block, as the block
        // itself used to keep: the call leaves the air it took.
        .pb(px(sizes::CODE_FONT))
        .gap_2()
        .child(block("Input:", move || {
            tool.with(|tool| tool.arguments.clone())
        }))
        .child(when(
            move || tool.with(|tool| !tool.output.is_empty()),
            move || block("Output:", move || tool.with(|tool| tool.output.clone())),
        ))
}

/// `title` over `value` as code, indented past the call's mark to its name: the
/// JSON laid out and highlighted where it is JSON.
fn block(
    title: &'static str,
    value: impl Fn() -> SharedString + Copy + 'static,
) -> impl IntoElement {
    view()
        .w_full()
        .pl(px(TOOL_TEXT_INSET))
        .gap_1()
        .child(text(title).text_xs().text_color(tokens::meta))
        .child(
            scroll()
                .w_full()
                .max_h(px(BLOCK_LINES * sizes::CODE_FONT * MESSAGE_LINE))
                .scrollbar(false)
                .child(markdown::data(value)),
        )
}

/// The files a call changed: each a card of its own, under a blank line that
/// keeps it off the row that names the call.
fn changes(tool: Memo<ToolRun>) -> impl IntoElement {
    view().w_full().gap_2().pt_1().child(each(
        move || tool.with(|tool| tool.changes.clone()),
        |change| change.path.clone(),
        move |change, _| change_card(change),
    ))
}

/// One file a call changed, as the diff card draws it.
fn change_card(change: ReadSignal<store::ToolChange>) -> impl IntoElement {
    diff::card(move || {
        change.with(|change| diff::Change {
            path: change.path.clone(),
            added: change.added,
            removed: change.removed,
            patch: change.diff.clone(),
        })
    })
}
