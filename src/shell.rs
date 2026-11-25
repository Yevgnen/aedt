//! The app shell: the session sidebar beside the conversation, no page logic.

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::{
    pages::{chat, sidebar},
    store,
};

pub(crate) fn shell() -> View {
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
                    let id = store::selected_session_id();
                    untracked(|| chat::pane(id).key(format!("chat-{id}")))
                })),
        )
}
