//! A call a reply made: its row, the file summary an edit shows, and the cards
//! of the files it changed.

use aedt_store::{self as store, ToolRun};
use aedt_ui::{MESSAGE_LINE, button, code, diff, icon, motion, sizes, tokens};
use nsh_base::theme;
use nsh_core::{LineHeight, prelude::*};

use super::Folds;

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
                    Some(SharedString::from(format!(
                        "{} result, {}",
                        tool.with(|tool| tool.name.clone()),
                        if shown() { "shown" } else { "hidden" }
                    )))
                })
                .w_full()
                .group()
                .flex_row()
                .items_center()
                .gap_2()
                .px_1()
                .py(px(2.0))
                .text_color(tokens::meta)
                .on_press(toggle)
                .child(icon::wrench(14.0))
                .child(
                    // The tool's name is prose about the call, not code in it:
                    // it reads at the size a reply's own words do, where the
                    // arguments and the file beside it are set in the code's.
                    text(move || tool.with(|tool| tool.name.clone()))
                        .flex_none()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view().flex_1().min_w(px(0.0)).child(
                        // An edit names the file it changed and how far, where a
                        // call that only runs something previews its arguments.
                        when(
                            move || tool.with(|tool| !tool.changes.is_empty()),
                            move || edit_summary(tool),
                        )
                        .otherwise(move || {
                            text(move || tool.with(|tool| tool.arguments.clone()))
                                .w_full()
                                .number_of_lines(1)
                                .font_family(code::MONO_FONT)
                                .text_size(px(sizes::CODE_FONT))
                                .text_color(tokens::meta)
                        }),
                    ),
                )
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
        .child(when(
            move || shown() && tool.with(|tool| !tool.changes.is_empty()),
            move || changes(tool),
        ))
        .child(when(
            move || shown() && tool.with(|tool| tool.changes.is_empty() && !tool.output.is_empty()),
            move || {
                code::block().child(
                    text(move || tool.with(|tool| tool.output.clone()))
                        .w_full()
                        .font_family(code::MONO_FONT)
                        .text_size(px(sizes::CODE_FONT))
                        .line_height(LineHeight::Multiple(MESSAGE_LINE))
                        .text_color(tokens::meta)
                        .number_of_lines(12),
                )
            },
        ))
}

/// What an edit did, where the row names the call: the first file it changed
/// and how far the call changed what it touched, in all.
fn edit_summary(tool: Memo<ToolRun>) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            text(move || {
                tool.with(|tool| {
                    tool.changes
                        .first()
                        .map(|change| change.path.clone())
                        .unwrap_or_default()
                })
            })
            .flex_1()
            .min_w(px(0.0))
            .number_of_lines(1)
            .font_family(code::MONO_FONT)
            .text_size(px(sizes::CODE_FONT))
            .text_color(tokens::meta),
        )
        .child(
            text(move || {
                tool.with(|tool| {
                    let added: usize = tool.changes.iter().map(|change| change.added).sum();
                    let removed: usize = tool.changes.iter().map(|change| change.removed).sum();

                    format!("+{added} −{removed}")
                })
            })
            .flex_none()
            .font_family(code::MONO_FONT)
            .text_size(px(sizes::CODE_FONT))
            .text_color(tokens::meta),
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
