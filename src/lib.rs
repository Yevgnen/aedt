//! Aedt: a desktop chat app in Rust. Sessions live in the sidebar, the selected
//! conversation fills the pane beside it, and replies stream from DeepSeek into
//! the conversation as they arrive.

mod llm;
mod pages;
mod settings;
mod shell;
mod store;

use aedt_ui::{MESSAGE_LINE, sizes, tokens};
use nsh_base::{TextViewPart, TextViewParts};
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

/// The least height the window may take, in points: the header and the
/// composer card with its status line keep their room, and a few exchanges
/// still read above the composer. Its width keeps its own freedom, so the pane
/// can narrow to any width the layout holds.
const MIN_HEIGHT: f32 = 560.0;

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
    // The composer's picker names the models a provider lists: ask for them
    // once, after the first pass, so the first reply does not wait for the
    // settings to be opened.
    effect(|| untracked(settings::load));

    // A code block draws its border in the token's border color, which reads as
    // an outline around its fill: the fill's own color leaves no outline while
    // the 1pt width stays. Its lines are as tight as a message's prose.
    provide_context(TextViewParts::new().part(TextViewPart::CodeBlock, |s| {
        s.bg(tokens::code)
            .border_color(tokens::code)
            .text_size(px(sizes::CODE_FONT))
            .line_height(LineHeight::Multiple(MESSAGE_LINE))
    }));

    let mut title_bar = TitleBarOptions::default();
    title_bar.style = TitleBarStyle::Hidden;
    let mut attributes = WindowAttributes::default();
    attributes.title_bar = Some(title_bar);
    attributes.min_size = Some(Some(size(px(0.0), px(MIN_HEIGHT))));

    app_window()
        .id("main")
        .title("Aedt")
        .attributes(attributes)
        .default_size(size(px(1320.0), px(880.0)))
        .content(shell::shell)
}
