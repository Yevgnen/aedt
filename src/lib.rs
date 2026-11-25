//! Aedt: a desktop chat app skeleton in Rust. Sessions live in the sidebar, the
//! selected conversation fills the pane beside it, and the composer echoes
//! placeholder model data while it changes.

mod pages;
mod shell;
mod store;
mod ui;

use nsh_core::{element::desktop::window as app_window, prelude::*};

/// The app's coding font, most preferred first. CoreText sets text in the
/// first installed family, so a machine without Berkeley Mono falls back to
/// another monospaced family instead of the proportional system font.
const MONO_FONT: &str = "Berkeley Mono, SF Mono, Menlo, monospace";

/// Points the `font-mono` token at [`MONO_FONT`]; code blocks, inline code
/// and any monospaced text read it. `cargo run` does not read `nsh.json`, so
/// the app sets the token itself; a host theme naming `font-mono` still wins.
fn set_mono_font() {
    let mut tokens = nsh_core::element::tokens().as_ref().clone();
    tokens.set_font("font-mono", MONO_FONT);
    set_tokens(tokens);
}

/// The app's UI set (`nsh.json` `"rust"`).
#[native_module("aedt/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}

/// The app's root: one window with the app's shell in it. Off the desktops
/// the shell fills the app's one surface, as a phone's window is the app.
#[component]
pub fn App() -> impl IntoElement {
    set_mono_font();

    app_window()
        .id("main")
        .title("Aedt")
        .default_size(size(px(1120.0), px(760.0)))
        .content(shell::shell)
}
