//! Aedt: a desktop chat app in Rust. Sessions live in the sidebar, the selected
//! conversation fills the pane beside it, and replies stream from DeepSeek into
//! the conversation as they arrive.

mod pages;
mod shell;

use aedt_ui::{code, sizes};
use nsh_core::{
    desktop::{TitleBarOptions, TitleBarStyle, WindowAttributes},
    element::desktop::window as app_window,
    prelude::*,
};

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
    tokens.set_font("font-mono", code::MONO_FONT);
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
    // The composer's picker names the models a provider lists: ask for them
    // once, after the first pass, so the first reply does not wait for the
    // settings to be opened.
    effect(|| untracked(aedt_settings::load));

    // A code block draws its border in the token's border color, which reads as
    // an outline around its fill: the fill's own color leaves no outline while
    // the 1pt width stays. Its lines are as tight as a message's prose.
    provide_context(code::parts());

    let mut title_bar = TitleBarOptions::default();
    title_bar.style = TitleBarStyle::Hidden;
    // The traffic lights are centered in the app's own top row, so they share a
    // line with the sidebar's brand and the conversation's header beside them.
    // They also move a little off the window's edge, and their 12 points keep
    // that centering exact.
    title_bar.height = Some(px(sizes::TOPBAR));
    title_bar.controls_offset = Some(point(
        px(sizes::SIDEBAR_INSET),
        px((sizes::TOPBAR - 12.0) / 2.0),
    ));
    let mut attributes = WindowAttributes::default();
    attributes.title_bar = Some(title_bar);
    attributes.min_size = Some(Some(size(px(0.0), px(sizes::WINDOW_MIN_H))));

    app_window()
        .id("main")
        .title("Aedt")
        .attributes(attributes)
        .default_size(size(px(sizes::WINDOW_W), px(sizes::WINDOW_H)))
        .content(shell::shell)
}
