//! Opening a conversation's workspace in Zed, and the mark that asks for it.

use std::process::{Command, ExitStatus};

use aedt_ui::{button, icon};
use nsh_core::prelude::*;

/// The mark that opens the conversation's workspace in Zed, while it works in
/// a folder. It stands beside the files panel's show or hide toggle, which is
/// the window's top right corner whether that panel shows or not.
pub(crate) fn button(
    workspace: impl Fn() -> Option<SharedString> + Copy + 'static,
) -> impl IntoElement {
    when(
        move || workspace().is_some(),
        move || {
            button::ghost_icon(icon::zed(14.0), "Open in Zed").on_press(move || {
                if let Some(path) = workspace() {
                    open(&path);
                }
            })
        },
    )
}

/// Hands `path` to Zed, in the editor's window for that folder. The handoff
/// runs on its own thread: the app's window waits on no other app, and the
/// child is reaped as the thread waits on it.
fn open(path: &str) {
    let path = path.to_owned();
    std::thread::spawn(move || {
        if let Err(error) = launch(&path) {
            eprintln!("zed: {error}");
        }
    });
}

/// Starts the editor on `path`. macOS opens the app bundle by its name, so the
/// folder comes up as the editor's workspace without the `zed` command line
/// installed; the other platforms ask the command.
fn launch(path: &str) -> std::io::Result<ExitStatus> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.args(["-a", "Zed"]);
        command
    };

    #[cfg(not(target_os = "macos"))]
    let mut command = Command::new("zed");

    command.arg(path).status()
}
