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
    // Whether the window is too narrow to hold both columns, as the shell's last
    // layout measured it.
    let narrow = signal(false);
    // Whether the sidebar shows: the toggle in its own top row hides it, and the
    // one the conversation's header then carries brings it back.
    let sidebar_open = signal(true);
    // While the window is too narrow for both columns, the reader's own showing
    // waits: the sidebar steps aside, and shows only where they ask for it
    // anyway. The setting stays as they left it, so a wider window shows it
    // again.
    let sidebar_peek = signal(false);
    let sidebar_shown = move || sidebar_open.get() && (!narrow.get() || sidebar_peek.get());
    let show_sidebar = move || {
        if narrow.get() {
            sidebar_open.set(true);
            sidebar_peek.set(true);
        } else {
            motion::set_panel(sidebar_open, true);
        }
    };
    // Once the window holds both columns again, the sidebar shows by the
    // reader's setting alone: the showing they asked for while it was narrow has
    // served.
    effect(move || {
        if !narrow.get() && sidebar_peek.get_untracked() {
            sidebar_peek.set(false);
        }
    });
    // Whether the files panel shows: the conversation's header opens it, and the
    // panel's own top row shuts it.
    let files_open = signal(false);
    // The folder the panel shows: the selected conversation's workspace, which
    // changes only when the reader moves to a conversation in another one.
    let workspace = computed(move || store::session_path(selected.get()));

    view()
        .flex_1()
        .flex_row()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .on_layout(move |frame: &Bounds<Points>| {
            let now = frame.size.width.0 < sizes::NARROW;
            if narrow.get_untracked() != now {
                // The panel fades where the width takes it away or gives it
                // back; the conversation's own frame is the drag's to move.
                motion::fade_panel();
                narrow.set(now);
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
                        chat::pane(id, sidebar_shown, show_sidebar, files_open)
                            .key(format!("chat-{id}"))
                    })
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
