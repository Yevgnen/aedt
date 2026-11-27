//! Opening a conversation's workspace in Zed, and the mark that asks for it.

use std::process::Command;

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
            button::ghost_icon(icon::zed(10.5), "Open in Zed").on_press(move || {
                if let Some(path) = workspace() {
                    open(&path);
                }
            })
        },
    )
}

/// How a folder is handed to the editor, in the order tried: the `zed` command
/// the editor installs, which knows the build the reader set up, then the app
/// bundles macOS opens by name. An editor installs itself as "Zed", or as "Zed
/// Preview" for the preview channel; opening the bundle also reaches an app
/// started from a launcher whose path lacks the command.
#[cfg(target_os = "macos")]
const LAUNCHES: &[(&str, &[&str])] = &[
    ("zed", &[]),
    ("open", &["-a", "Zed"]),
    ("open", &["-a", "Zed Preview"]),
];

/// How a folder is handed to the editor away from macOS: the `zed` command.
#[cfg(not(target_os = "macos"))]
const LAUNCHES: &[(&str, &[&str])] = &[("zed", &[])];

/// Hands `path` to Zed, in the editor's window for that folder. The handoff
/// runs on its own thread: the app's window waits on no other app, and the
/// child is reaped as the thread waits on it.
fn open(path: &str) {
    let path = path.to_owned();
    std::thread::spawn(move || {
        if let Err(complaint) = launch(&path) {
            eprintln!("zed: failed to open {path:?}: {complaint}");
        }
    });
}

/// Starts the editor on `path` by the first launch that answers. Each launch
/// is waited on, so one that fails says why before the next is tried.
fn launch(path: &str) -> Result<(), String> {
    let mut complaint = String::new();
    for &(program, args) in LAUNCHES {
        match Command::new(program).args(args).arg(path).output() {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stderr = stderr.trim();
                complaint = if stderr.is_empty() {
                    format!("{program} failed: {}", output.status)
                } else {
                    stderr.to_owned()
                };
            }
            Err(error) => complaint = format!("{program}: {error}"),
        }
    }

    Err(complaint)
}
