//! The app shell: the session sidebar beside the conversation, no page logic.

use aedt_store as store;
use nsh_base::theme;
use nsh_core::prelude::*;

use crate::pages::{chat, settings, sidebar};

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

    view()
        .flex_1()
        .flex_row()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
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
                    untracked(|| chat::pane(id, sidebar_open).key(format!("chat-{id}")))
                })),
        )
        // The settings, over everything while they are open.
        .child(settings::dialog(settings_open))
}
