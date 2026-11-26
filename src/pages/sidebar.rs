//! The session list: a persistent sidebar beside the conversation, its
//! conversations grouped under the workspace each works in.

use aedt_store as store;
use aedt_ui::{avatar, button, icon, list_item, menu, motion, sizes};
use nsh_base::theme;
use nsh_core::{element::desktop::title_bar, file_dialog::OpenDialog, prelude::*};

/// The session list, with the action that opens the settings as its foot.
pub(crate) fn sidebar(open_settings: impl Fn() + Copy + 'static) -> View {
    view()
        .w(px(sizes::SIDEBAR))
        .h_full()
        .bg(move || theme().sidebar)
        .border_r_1()
        .border_r_color(move || theme().sidebar_border)
        .child(brand())
        .child(header())
        .child(sessions())
        .child(footer(open_settings))
}

/// The list's heading, and the action that adds a workspace: a folder its
/// conversations then work in.
fn header() -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap_2()
        .px_3()
        .pb_2()
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

fn brand() -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        // The traffic lights own the whole title bar band on the left edge, so
        // the brand starts right below it: no extra padding above.
        .pt(move || Length::from(title_bar().top))
        .pb_4()
        .window_drag(true)
        .child(avatar::avatar("A"))
        .child(
            text("Aedt")
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().sidebar_foreground),
        )
}

/// Every group of conversations: each workspace's under its name, the ones
/// held in no workspace last under `Ungrouped`.
fn sessions() -> impl IntoElement {
    scroll().flex_1().child(each(
        store::groups,
        |group| group.workspace.as_ref().map_or(0, |workspace| workspace.id),
        |group, _| group_view(group),
    ))
}

/// One group: its name over the conversations held in it, with the action
/// that starts another there once the pointer is over the row.
fn group_view(group: ReadSignal<store::Group>) -> impl IntoElement {
    let workspace =
        move || group.with(|group| group.workspace.as_ref().map(|workspace| workspace.id));
    let name = move || {
        group.with(|group| match &group.workspace {
            Some(workspace) => workspace.name.clone(),
            None => SharedString::from("Ungrouped"),
        })
    };

    view()
        .w_full()
        .gap_1()
        .pb_1()
        .child(
            view()
                .w_full()
                .group()
                .flex_row()
                .items_center()
                .gap_2()
                .min_h(px(sizes::LIST_ROW))
                .px(px(sizes::LIST_ROW_INSET))
                .text_color(move || theme().sidebar_foreground)
                .child(icon::folder_open(14.0))
                .child(text(name).flex_1().text_size(px(13.0)))
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
                        .on_press(move || store::create_session_in(workspace()))
                        .child(icon::plus(14.0)),
                ),
        )
        // The rows sit against one another: their hover fills touch, as a
        // column of them reads, where a gap would cut each into a block.
        .child(view().w_full().pl(px(24.0)).child(each(
            move || group.with(|group| group.sessions.clone()),
            |session| session.id,
            |session, _| session_row(session),
        )))
}

/// One conversation: its title, selected while it is the one on show, and the
/// actions its row offers at the end while the pointer is over it.
fn session_row(session: ReadSignal<store::SessionSummary>) -> impl IntoElement {
    let id = move || session.with(|session| session.id);
    let title = move || session.with(|session| session.title.clone());

    view()
        .w_full()
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
