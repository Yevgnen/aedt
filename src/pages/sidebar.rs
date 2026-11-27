//! The session list: a persistent sidebar beside the conversation, its
//! conversations grouped under the workspace each works in.

use aedt_store as store;
use aedt_ui::{button, icon, list_item, menu, motion, sizes, state};
use nsh_base::{collapsible, theme};
use nsh_core::{file_dialog::OpenDialog, prelude::*};

/// The session list, with the action that opens the settings as its foot, and
/// the one in its top row that hides it.
pub(crate) fn sidebar(open_settings: impl Fn() + Copy + 'static, open: Signal<bool>) -> View {
    view()
        .w(px(sizes::SIDEBAR))
        .h_full()
        .bg(move || theme().sidebar)
        .border_r_1()
        .border_r_color(move || theme().sidebar_border)
        .child(brand(open))
        .child(header())
        .child(sessions())
        .child(footer(open_settings))
}

/// The list's heading, and the action that adds a workspace: a folder its
/// conversations then work in.
fn header() -> impl IntoElement {
    view()
        .h(px(sizes::HEADING))
        .flex_row()
        .items_center()
        .gap_2()
        .pl(px(sizes::SIDEBAR_INSET))
        .pr_3()
        .child(
            text("Workspaces")
                .flex_1()
                .text_sm()
                .text_color(move || theme().muted_foreground),
        )
        .child(
            button::ghost_icon(icon::folder_plus(14.0), "Add a workspace")
                // A wash of the accent, as the row under it takes: the muted one
                // the other icon buttons use is lost on the sidebar's own tone.
                .hover(|s| s.bg(move || theme().accent))
                .on_press(add_workspace),
        )
}

/// Asks the system for a folder, and adds the one picked as a workspace.
fn add_workspace() {
    Owner::detached().run(|| {
        spawn(async {
            let dialog = OpenDialog::folders().button_label("Add workspace");
            if let Ok(Some(paths)) = pick_paths(dialog).await
                && let Some(path) = paths.first()
            {
                store::add_workspace(&path.to_string_lossy());
            }
        });
    });
}

/// The sidebar's foot, at the bottom edge it belongs to: the menu of the
/// app-wide commands.
fn footer(open_settings: impl Fn() + Copy + 'static) -> impl IntoElement {
    view().px_3().pb_3().child(menu::menu(
        menu::trigger(icon::more(14.0), "More"),
        Placement::Top,
        Align::Start,
        menu::item(icon::settings(14.0), "Settings").on_select(open_settings),
    ))
}

fn brand(open: Signal<bool>) -> impl IntoElement {
    view()
        .h(px(sizes::TOPBAR))
        .flex_row()
        .items_center()
        // The toggle sits at the sidebar's right edge, where the conversation's
        // own copy of it stands while the sidebar is away, so it does not move.
        .justify_end()
        .pr_3()
        .window_drag(true)
        .child(
            button::ghost_icon(icon::panel_left(14.0), "Hide the sidebar")
                .on_press(move || motion::set_panel(open, false)),
        )
}

/// The air the list keeps from the sidebar's edges: a row's wash stops short
/// of them, so the pane reads as a column standing inside it.
const LIST_EDGE: f32 = 8.0;

/// Every group of conversations: each workspace's under its name, the ones
/// held in no workspace last under `Ungrouped`.
fn sessions() -> impl IntoElement {
    scroll()
        .flex_1()
        .scrollbar(false)
        .child(view().w_full().px(px(LIST_EDGE)).child(each(
            store::groups,
            |group| group.workspace.as_ref().map_or(0, |workspace| workspace.id),
            |group, _| group_view(group),
        )))
}

/// One group: its name over the conversations held in it, with the action
/// that starts another there once the pointer is over the row. Pressing the
/// row folds the conversations away, and pressing it again brings them back.
fn group_view(group: ReadSignal<store::Group>) -> impl IntoElement {
    let workspace =
        move || group.with(|group| group.workspace.as_ref().map(|workspace| workspace.id));
    let name = move || {
        group.with(|group| match &group.workspace {
            Some(workspace) => workspace.name.clone(),
            None => SharedString::from("Ungrouped"),
        })
    };
    // The workspace this row stands for: the ones held in none have no folder
    // to take out of the sidebar.
    let workspace_id =
        group.with_untracked(|group| group.workspace.as_ref().map(|workspace| workspace.id));
    // The conversations show until the reader folds them: the list is what
    // the sidebar is for.
    let open = signal(true);
    let fold = move || open.update(|open| *open = !*open);

    let header = view()
        .w_full()
        .group()
        .flex_row()
        .items_center()
        .gap_2()
        .min_h(px(sizes::LIST_ROW))
        // The list already keeps `LIST_EDGE` from the edges; the rest of the
        // row's inset comes from here, so the name and the action keep their
        // places on the sidebar's lines.
        .pl(px(sizes::SIDEBAR_INSET - LIST_EDGE))
        .pr(px(sizes::LIST_ROW_INSET - LIST_EDGE))
        .rounded(move || theme().radius)
        .text_color(move || theme().sidebar_foreground)
        .role(AccessibilityRole::Button)
        .accessibility_state(move || AccessibilityState {
            expanded: Some(open.get()),
            ..AccessibilityState::default()
        })
        .on_press(fold)
        .hover(|s| s.bg(move || theme().muted))
        .focus_visible(state::focus_ring)
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(
            // The folder says whether the conversations show: open while
            // they do, and with its minus while they are folded away.
            when(move || open.get(), || icon::folder_open(14.0))
                .otherwise(|| icon::folder_minus(14.0)),
        )
        .child(text(name).flex_1().text_size(px(sizes::TEXT_COMPACT)))
        .when_some(workspace_id, |header, id| {
            header.child(workspace_actions(id))
        })
        .child(
            // The only place a conversation starts, shown while the row
            // is under the pointer and washed when the pointer is on it.
            button::bare()
                .accessibility_label("Start a conversation")
                .size(px(sizes::META_ACTION))
                .items_center()
                .justify_center()
                .rounded_md()
                .opacity(0.0)
                .group_hover(|s| s.opacity(1.0))
                .hover(|s| s.bg(move || theme().muted))
                .transition_all()
                .duration(motion::row_duration())
                .ease_out()
                // A conversation added here opens the group it joins, so
                // it shows at once rather than behind a folded row.
                .on_press(move || {
                    open.set(true);
                    store::create_session_in(workspace());
                })
                .child(icon::plus(14.0)),
        );

    collapsible()
        .open(open)
        // The conversations unfold and fold again: the region springs from
        // zero to their height and back, and what follows moves with it.
        .reveal(true)
        .w_full()
        .pb_1()
        .child(header)
        // The rows are bare here: each carries its own air above it
        // ([`session_row`]), so a group with none adds no height at all and
        // folding it leaves what follows where it was.
        .content(view().w_full().pl(px(24.0 - LIST_EDGE)).child(each(
            move || group.with(|group| group.sessions.clone()),
            |session| session.id,
            |session, _| session_row(session),
        )))
}

/// The actions of the workspace `id`, at the end of its row while the pointer
/// is over it: taking it out of the sidebar, its folder left where it is.
fn workspace_actions(id: u32) -> impl IntoElement {
    menu::menu(
        button::bare()
            .accessibility_label("Workspace actions")
            .size(px(sizes::META_ACTION))
            .items_center()
            .justify_center()
            .rounded_md()
            .opacity(0.0)
            .group_hover(|s| s.opacity(1.0))
            // The pointer leaves the row to pick an item; the menu marks the
            // button it opens, which keeps it shown while it is open.
            .expanded(|s| s.opacity(1.0))
            .hover(|s| s.bg(move || theme().muted))
            .transition_all()
            .duration(motion::row_duration())
            .ease_out()
            .child(icon::more(14.0)),
        Placement::Bottom,
        Align::End,
        menu::item(icon::trash(14.0), "Remove").on_select(move || store::remove_workspace(id)),
    )
}

/// One conversation: its title, selected while it is the one on show, and the
/// actions its row offers at the end while the pointer is over it.
fn session_row(session: ReadSignal<store::SessionSummary>) -> impl IntoElement {
    let id = move || session.with(|session| session.id);
    let title = move || session.with(|session| session.title.clone());

    view()
        .w_full()
        // The air above the row keeps its wash clear of the group's own row
        // and of the conversation above it; a margin rather than a gap on the
        // list, so the row's actions still centre on the row itself.
        .mt_1()
        .group()
        .child(
            list_item::nav(title, move || {
                Some(id()) == Some(store::selected_session_id())
            })
            .accessibility_label(move || Some(title()))
            // Room at the end for the actions, which float over the row.
            .pr(px(sizes::LIST_ROW_INSET + sizes::META_ACTION))
            .on_press(move || store::select_session(id())),
        )
        .child(session_actions(id))
}

/// The actions of the conversation `id`, at the end of its row while the
/// pointer is over it: putting it away, and taking it out for good.
fn session_actions(id: impl Fn() -> u32 + Copy + 'static) -> impl IntoElement {
    view()
        .absolute()
        .top(px(0.0))
        .bottom(px(0.0))
        .right(px(sizes::LIST_ROW_INSET))
        .justify_center()
        .child(menu::menu(
            button::bare()
                .accessibility_label("Session actions")
                .size(px(sizes::META_ACTION))
                .items_center()
                .justify_center()
                .rounded_md()
                .opacity(0.0)
                .group_hover(|s| s.opacity(1.0))
                // The pointer leaves the row to pick an item; the menu marks
                // the button it opens, which keeps it shown while it is open.
                .expanded(|s| s.opacity(1.0))
                .hover(|s| s.bg(move || theme().muted))
                .transition_all()
                .duration(motion::row_duration())
                .ease_out()
                .child(icon::more(14.0)),
            Placement::Bottom,
            Align::End,
            view()
                .child(
                    menu::item(icon::archive(14.0), "Archive")
                        .on_select(move || store::archive_session(id())),
                )
                .child(
                    menu::item(icon::trash(14.0), "Delete")
                        .on_select(move || store::delete_session(id())),
                ),
        ))
}
