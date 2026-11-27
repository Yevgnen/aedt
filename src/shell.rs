//! The app shell: the session sidebar beside the conversation, no page logic.

use aedt_store as store;
use aedt_ui::sizes;
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
    // Whether the sidebar shows: the toggle in its own top row hides it, and the
    // one the conversation's header then carries brings it back.
    let sidebar_open = signal(true);
    // Whether the files panel shows: the conversation's header opens it, and the
    // panel's own top row shuts it.
    let files_open = signal(false);
    // The folder the panel shows: the selected conversation's workspace, which
    // changes only when the reader moves to a conversation in another one.
    let workspace = computed(move || store::session_path(selected.get()));
    // Whether the window is too narrow to hold both columns, as the shell's last
    // layout measured it.
    let narrow = signal(false);
    // A window too narrow for both keeps the sidebar out of the way: the
    // conversation is what a narrow window is for, and the toggle in its header
    // brings the list back when the reader asks for it.
    effect(move || {
        if narrow.get() && sidebar_open.get() {
            sidebar_open.set(false);
        }
    });

    view()
        .flex_1()
        .flex_row()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .on_layout(move |frame: &Bounds<Points>| {
            let now = frame.size.width.0 < sizes::NARROW;
            if narrow.get_untracked() != now {
                narrow.set(now);
            }
        })
        .child(when(
            move || sidebar_open.get(),
            move || sidebar::sidebar(open_settings, sidebar_open),
        ))
        .child(
            view()
                .flex_1()
                .min_w(px(0.0))
                // A new conversation replaces the pane and its state.
                .child(dynamic(move || {
                    let id = selected.get();
                    untracked(|| chat::pane(id, sidebar_open, files_open).key(format!("chat-{id}")))
                })),
        )
        // The files of the conversation's folder, beside it while they show.
        .child(when(
            move || files_open.get(),
            move || files::panel(workspace, files_open),
        ))
        // The settings, over everything while they are open.
        .child(settings::dialog(settings_open))
}
