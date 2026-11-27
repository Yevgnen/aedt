//! A message's prose: the Markdown a reply answers in, read at the size and
//! density of the conversation around it.

use nsh_base::{MarkdownOptions, TextViewClasses, TextViewPart, TextViewParts, text_view, theme};
use nsh_core::{LineHeight, prelude::*};

use crate::{MESSAGE_LINE, code};

/// The parts a reply's prose draws with: a code block as [`code`] sets it, and
/// a table's rows as close as the app's own lists — its cells' air above and
/// below is a list row's, and its lines are as tight as the prose around them.
pub fn parts() -> TextViewParts {
    code::parts()
        .part(TextViewPart::TableCell, |s| s.py(px(5.0)))
        .part(TextViewPart::Table, |s| {
            s.line_height(LineHeight::Multiple(MESSAGE_LINE))
        })
}

/// `value` as the block a reply's code takes, in the look and the highlighter
/// the reply's own code blocks read with: its JSON laid out when it parses as
/// JSON, exactly as it came otherwise.
pub fn data(value: impl Fn() -> SharedString + Copy + 'static) -> impl IntoElement {
    text_view()
        .w_full()
        .markdown(move || fenced(&value()))
        .classes(TextViewClasses {
            link: Some("text-link".into()),
            ..TextViewClasses::default()
        })
        .selectable(true)
        .code_block_scrollbar(false)
        .gap_2()
        .text_sm()
        .line_height(LineHeight::Multiple(MESSAGE_LINE))
        .text_color(move || theme().foreground)
}

/// `value` as Markdown's one code block: its JSON laid out under the fence that
/// tells the highlighter it is JSON, and as it came, plain, when it is not.
fn fenced(value: &str) -> SharedString {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return SharedString::from(format!("```\n{value}\n```"));
    };
    let laid_out = serde_json::to_string_pretty(&json).unwrap_or_else(|_| value.to_owned());

    SharedString::from(format!("```json\n{laid_out}\n```"))
}

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
