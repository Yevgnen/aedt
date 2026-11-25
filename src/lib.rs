//! Aedt: a desktop chat app in Rust. Sessions live in the sidebar, the selected
//! conversation fills the pane beside it, and replies stream from DeepSeek into
//! the conversation as they arrive.

mod llm;
mod pages;
mod shell;
mod store;
mod ui;

use nsh_base::{TextViewPart, TextViewParts, theme};
use nsh_core::{
    LineHeight,
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

/// A code block's fill, the token `code` (the theme's `muted` when it leaves
/// that out). It is lighter than `muted`, so a block reads as a soft inset
/// rather than a gray box, and its border takes the same color.
fn code_fill() -> AdaptiveColor {
    theme().color("code").unwrap_or_else(|| theme().muted)
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

    // A code block draws its border in the token's border color, which reads as
    // an outline around its fill: the fill's own color leaves no outline while
    // the 1pt width stays. Its lines are as tight as a message's prose.
    provide_context(TextViewParts::new().part(TextViewPart::CodeBlock, |s| {
        s.bg(code_fill)
            .border_color(code_fill)
            .text_size(px(ui::sizes::CODE_FONT))
            .line_height(LineHeight::Multiple(ui::MESSAGE_LINE))
    }));

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
