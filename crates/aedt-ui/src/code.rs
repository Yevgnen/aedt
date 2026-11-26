//! Code on screen: the family it is set in, the block a stretch of it sits on,
//! and the parts a Markdown code block draws with.

use nsh_base::{TextViewPart, TextViewParts, theme};
use nsh_core::{LineHeight, prelude::*};

use crate::{MESSAGE_LINE, sizes, tokens};

/// The app's coding font, most preferred first. CoreText sets text in the first
/// installed family, so a machine without Berkeley Mono falls back to another
/// monospaced family instead of the proportional system font.
pub const MONO_FONT: &str = "Berkeley Mono, SF Mono, Menlo, monospace";

/// The parts a Markdown code block draws with: its border in the token's own
/// color, which reads as an outline around its fill — the fill's color leaves
/// no outline while the 1pt width stays — and its lines as tight as a message's
/// prose.
pub fn parts() -> TextViewParts {
    TextViewParts::new().part(TextViewPart::CodeBlock, |s| {
        s.bg(tokens::code)
            .border_color(tokens::code)
            .text_size(px(sizes::CODE_FONT))
            .line_height(LineHeight::Multiple(MESSAGE_LINE))
    })
}

/// A block a stretch of code sits on: the token's fill, the app's radius, and a
/// blank line under the last one as tall as the text, so the block ends where
/// its content did rather than against its edge.
pub fn block() -> View {
    view()
        .w_full()
        .rounded(move || theme().radius)
        .bg(tokens::code)
        .px_2()
        .py_1()
        .pb(px(sizes::CODE_FONT))
}
