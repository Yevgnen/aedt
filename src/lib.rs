//! Aedt: a desktop chat app skeleton in Rust. Sessions live in the sidebar, the
//! selected conversation fills the pane beside it, and the composer echoes
//! placeholder model data while it changes.

mod pages;
mod shell;
mod store;
mod ui;

use nsh_core::{
    desktop::{TitleBarOptions, TitleBarStyle, WindowAttributes},
    element::desktop::window as app_window,
    prelude::*,
};

/// The app's coding font, most preferred first. CoreText sets text in the
/// first installed family, so a machine without Berkeley Mono falls back to
/// another monospaced family instead of the proportional system font.
const MONO_FONT: &str = "Berkeley Mono, SF Mono, Menlo, monospace";

/// The app's theme, the same file `nsh.json` names for `nsh run`.
const THEME: &str = include_str!("../theme.json");

/// Applies the theme and the coding font. `cargo run` does not read
/// `nsh.json`, so the app sets its own tokens at startup; a host theme naming
/// the same tokens still wins.
fn apply_theme() {
    let current = nsh_core::element::tokens().as_ref().clone();
    let mut tokens = match nsh_base::tokens_of(Some(THEME)) {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("theme: {error}");
            current
        }
    };
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
    apply_theme();

    let mut title_bar = TitleBarOptions::default();
    title_bar.style = TitleBarStyle::Hidden;
    let mut attributes = WindowAttributes::default();
    attributes.title_bar = Some(title_bar);

    app_window()
        .id("main")
        .title("Aedt")
        .attributes(attributes)
        .default_size(size(px(1320.0), px(880.0)))
        .content(shell::shell)
}
