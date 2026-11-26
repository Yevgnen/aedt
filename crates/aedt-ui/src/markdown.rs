//! A message's prose: the Markdown a reply answers in, read at the size and
//! density of the conversation around it.

use nsh_base::{MarkdownOptions, TextViewClasses, text_view, theme};
use nsh_core::{LineHeight, prelude::*};

use crate::MESSAGE_LINE;

/// `markdown`, as a reply reads it. `streaming` says whether it is still
/// arriving: while it is, the text view shows its open syntax as closed and
/// again only the blocks that changed.
pub fn body(
    markdown: impl Fn() -> SharedString + Copy + 'static,
    streaming: impl Fn() -> bool + Copy + 'static,
) -> impl IntoElement {
    text_view()
        .markdown(markdown)
        .classes(TextViewClasses {
            link: Some("text-link".into()),
            ..TextViewClasses::default()
        })
        .options(move || MarkdownOptions {
            streaming: streaming(),
            ..MarkdownOptions::default()
        })
        .selectable(true)
        .code_block_scrollbar(false)
        .gap_2()
        .text_sm()
        .line_height(LineHeight::Multiple(MESSAGE_LINE))
        .text_color(move || theme().foreground)
}
