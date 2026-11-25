//! Aedt: a desktop chat app skeleton in Rust. Sessions live in the sidebar, the
//! selected conversation fills the pane beside it, and the composer echoes
//! placeholder model data while it changes.

mod pages;
mod shell;
mod store;
mod ui;

use nsh_core::{element::desktop::window as app_window, prelude::*};

/// The app's UI set (`nsh.json` `"rust"`).
#[native_module("aedt/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}

/// The app's root: one window with the app's shell in it. Off the desktops
/// the shell fills the app's one surface, as a phone's window is the app.
#[component]
pub fn App() -> impl IntoElement {
    app_window()
        .id("main")
        .title("Aedt")
        .default_size(size(px(1120.0), px(760.0)))
        .content(shell::shell)
}
