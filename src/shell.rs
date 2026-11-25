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

    view()
        .flex_1()
        .flex_row()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .child(sidebar::sidebar())
        .child(
            view()
                .flex_1()
                .min_w(px(0.0))
                // A new conversation replaces the pane and its state.
                .child(dynamic(move || {
                    let id = selected.get();
                    untracked(|| chat::pane(id).key(format!("chat-{id}")))
                })),
        )
        // The settings, over everything while they are open.
        .child(settings::dialog())
}
