//! The app shell: the session sidebar beside the conversation, no page logic.

use aedt_store as store;
use aedt_ui::{motion, sizes};
use nsh_base::theme;
use nsh_core::prelude::*;

use crate::pages::{chat, files, settings, sidebar};

pub(crate) fn shell() -> View {
    // Whose pane shows: a memo of the selected id, so a reply streaming into
    // the current session (which writes the store on every token) does not
    // make the pane again — only selecting another session does.
    let selected = computed(store::selected_session_id);
    // Where the settings show. Opening them also asks every provider that has
    // not listed its models yet, so what the app can know fills in unasked.
    let settings_open = signal(false);
    let open_settings = move || {
        aedt_settings::load();
        settings_open.set(true);
    };
    // Command+, opens the settings, as the sidebar's menu item does.
    on_key("mod+,", move |_| open_settings());
    // Command+Backspace deletes the session on show, as its row's menu does.
    // The dialog keeps the key from reaching the sessions behind it.
    on_key("mod+backspace", move |_| {
        if !settings_open.get() {
            store::delete_session(selected.get());
        }
    });
    // Whether the window has room for each panel beside the conversation, as
    // the shell's last layout measured it: the sidebar steps aside first, and
    // the files panel follows it when the width leaves even less.
    let sidebar_narrow = signal(false);
    let files_narrow = signal(false);
    // The reader's own showing of each side panel: the one's toggle in its own
    // top row hides it, and the conversation's header brings it back; the
    // other's header opens it and its own top row shuts it.
    let sidebar_open = signal(true);
    let files_open = signal(false);
    // While the window is too narrow for a panel beside the conversation, the
    // reader's own showing waits: the panel steps aside, and shows only where
    // they ask for it anyway. The setting stays as they left it, so a wider
    // window shows it again.
    let sidebar_peek = signal(false);
    let files_peek = signal(false);
    let sidebar_shown = move || sidebar_open.get() && (!sidebar_narrow.get() || sidebar_peek.get());
    let files_shown = move || files_open.get() && (!files_narrow.get() || files_peek.get());
    let show_sidebar = move || {
        if sidebar_narrow.get() {
            sidebar_open.set(true);
            sidebar_peek.set(true);
        } else {
            motion::set_panel(sidebar_open, true);
        }
    };
    let show_files = move || {
        if files_narrow.get() {
            files_open.set(true);
            files_peek.set(true);
        } else {
            motion::set_panel(files_open, true);
        }
    };
    // Once the window holds a panel again, it shows by the reader's setting
    // alone: the showing they asked for while it was narrow has served.
    effect(move || {
        if !sidebar_narrow.get() && sidebar_peek.get_untracked() {
            sidebar_peek.set(false);
        }
        if !files_narrow.get() && files_peek.get_untracked() {
            files_peek.set(false);
        }
    });
    // The folder the panel shows: the selected conversation's workspace, which
    // changes only when the reader moves to a conversation in another one.
    let workspace = computed(move || store::session_path(selected.get()));

    view()
        .flex_1()
        .flex_row()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .on_layout(move |frame: &Bounds<Points>| {
            let width = frame.size.width.0;
            // What the sidebar takes while the reader keeps it open. The files
            // panel counts it and leaves first, so no width takes both at once
            // and leaves the conversation squeezed between them.
            let sidebar = if sidebar_open.get_untracked() {
                sizes::SIDEBAR
            } else {
                0.0
            };
            let files_now = width - sidebar - sizes::SIDEBAR < sizes::ROOM;
            let sidebar_now = width - sizes::SIDEBAR < sizes::ROOM;
            if sidebar_narrow.get_untracked() != sidebar_now
                || files_narrow.get_untracked() != files_now
            {
                // Each panel comes or goes with the motion the reader's own
                // toggle uses.
                motion::animate_panel();
                sidebar_narrow.set(sidebar_now);
                files_narrow.set(files_now);
            }
        })
        .child(when(sidebar_shown, move || {
            sidebar::sidebar(open_settings, sidebar_open)
        }))
        .child(
            view()
                .flex_1()
                .min_w(px(0.0))
                // A new conversation replaces the pane and its state.
                .child(dynamic(move || {
                    let id = selected.get();
                    untracked(|| {
                        chat::pane(id, sidebar_shown, show_sidebar, files_shown, show_files)
                            .key(format!("chat-{id}"))
                    })
                })),
        )
        // The files of the conversation's folder, beside it while they show.
        .child(when(files_shown, move || {
            files::panel(workspace, files_open)
        }))
        // The settings, over everything while they are open.
        .child(settings::dialog(settings_open))
}
