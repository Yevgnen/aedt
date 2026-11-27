//! Syntax colours for a diff's lines: the language is the file's, and the inks
//! are the ones a reply's code blocks take, so the two read alike.

use std::sync::OnceLock;

use nsh_core::prelude::*;
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

/// A kind of token, by what it means rather than by the grammar that made it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Comment,
    Keyword,
    String,
    Number,
    Title,
    Attr,
    Built,
}

impl Token {
    /// Its ink: the light and dark pair a code block's highlighter uses.
    pub fn color(self) -> AdaptiveColor {
        match self {
            Self::Comment => AdaptiveColor::rgb(0x6e7781, 0x7d7a72),
            Self::Keyword => AdaptiveColor::rgb(0x8250df, 0xc4a7ff),
            Self::String => AdaptiveColor::rgb(0x116329, 0x9fd68a),
            Self::Number => AdaptiveColor::rgb(0x953800, 0xf5b27a),
            Self::Title => AdaptiveColor::rgb(0x0550ae, 0x82b8ff),
            Self::Attr => AdaptiveColor::rgb(0x7d4e00, 0xf0c674),
            Self::Built => AdaptiveColor::rgb(0x0a7285, 0x6fd3d8),
        }
    }

    /// Whether it is set in italics.
    pub fn italic(self) -> bool {
        self == Self::Comment
    }
}

/// The tokens of `lines` in the language `path` names: for each line, the byte
/// ranges that take a token's ink. A file whose language is unknown has none,
/// and reads plain.
pub fn spans(path: &str, lines: &[&str]) -> Vec<Vec<(usize, usize, Token)>> {
    let Some(syntax) = syntax_of(path) else {
        return vec![Vec::new(); lines.len()];
    };
    let mut state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();

    lines
        .iter()
        .map(|line| line_spans(&mut state, &mut stack, line))
        .collect()
}

/// The tokens of one line, parsed after the lines before it.
fn line_spans(
    state: &mut ParseState,
    stack: &mut ScopeStack,
    line: &str,
) -> Vec<(usize, usize, Token)> {
    let mut spans = Vec::new();
    let Ok(ops) = state.parse_line(line, syntaxes()) else {
        return spans;
    };

    let mut at = 0;
    for (offset, op) in ops {
        if offset > at
            && let Some(token) = token_of(stack)
        {
            spans.push((at, offset, token));
        }
        if stack.apply(&op).is_err() {
            break;
        }
        at = offset;
    }
    if at < line.len()
        && let Some(token) = token_of(stack)
    {
        spans.push((at, line.len(), token));
    }

    spans
}

/// The token a scope stack ends in: its most specific scope that names one.
fn token_of(stack: &ScopeStack) -> Option<Token> {
    stack
        .as_slice()
        .iter()
        .rev()
        .find_map(|scope| token_named(&scope.build_string()))
}

/// The token a scope names, by the prefixes grammars share.
fn token_named(scope: &str) -> Option<Token> {
    Some(if scope.starts_with("comment") {
        Token::Comment
    } else if scope.starts_with("keyword") || scope.starts_with("storage") {
        Token::Keyword
    } else if scope.starts_with("string") {
        Token::String
    } else if scope.starts_with("constant") {
        Token::Number
    } else if scope.starts_with("entity.name.function")
        || scope.starts_with("entity.name.type")
        || scope.starts_with("entity.name.class")
        || scope.starts_with("entity.name.tag")
    {
        Token::Title
    } else if scope.starts_with("variable.parameter")
        || scope.starts_with("entity.other.attribute-name")
        || scope.starts_with("variable.other.member")
        || scope.starts_with("variable.other.property")
    {
        Token::Attr
    } else if scope.starts_with("support") {
        Token::Built
    } else {
        return None;
    })
}

/// The syntax set every lookup shares, loaded once.
fn syntaxes() -> &'static SyntaxSet {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAXES.get_or_init(two_face::syntax::extra_newlines)
}

/// The syntax `path` names by its extension, when the set has one.
fn syntax_of(path: &str) -> Option<&'static SyntaxReference> {
    let (_, extension) = path.rsplit_once('.')?;
    syntaxes().find_syntax_by_extension(extension)
}
