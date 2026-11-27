//! A file a call changed, drawn as a card: the file's name and how far the call
//! changed it, over the lines it changed. The untouched lines between two hunks
//! are left out of the card and counted on a line of their own.

use nsh_base::theme;
use nsh_core::{LineHeight, prelude::*};

use crate::{MESSAGE_LINE, code, highlight, sizes, tokens};

/// One file a call changed, as the card shows it.
#[derive(Clone, PartialEq)]
pub struct Change {
    /// The file, as the call named it.
    pub path: SharedString,
    /// Lines it added.
    pub added: usize,
    /// Lines it removed.
    pub removed: usize,
    /// The change as a unified diff: empty for a change the tool did not diff.
    pub patch: SharedString,
}

/// One row of the card: a line of the change, or the lines left out before it.
#[derive(Clone, PartialEq)]
enum Item {
    Line {
        /// Its place in the card, which keys its row.
        at: usize,
        /// What it does: `+` added, `-` removed, anything else kept.
        sign: char,
        text: SharedString,
        /// The bytes of `text` that changed, when the line paired with one on
        /// the other side and only part of it did.
        changed: Option<(usize, usize)>,
        /// The bytes that take a syntax token's ink.
        tokens: Vec<(usize, usize, highlight::Token)>,
    },
    /// The untouched lines between this hunk and the one before it.
    Gap { at: usize, lines: usize },
}

/// The card of one file a call changed.
pub fn card(change: impl Fn() -> Change + Copy + 'static) -> View {
    let change = computed(change);
    let items = computed(move || change.with(|change| items_of(&change.path, &change.patch)));

    view()
        .w_full()
        .rounded(move || theme().radius)
        // A line's fill is square: the card's corners keep it.
        .overflow_hidden()
        .bg(tokens::code)
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap_2()
                .bg(move || theme().muted)
                .px_2()
                .py_1()
                .child(
                    text(move || change.with(|change| change.path.clone()))
                        .flex_1()
                        .min_w(px(0.0))
                        .number_of_lines(1)
                        .font_family(code::MONO_FONT)
                        .text_size(px(sizes::CODE_FONT))
                        .text_color(tokens::meta),
                )
                .child(
                    text(move || {
                        change.with(|change| format!("+{} −{}", change.added, change.removed))
                    })
                    .flex_none()
                    .font_family(code::MONO_FONT)
                    .text_size(px(sizes::CODE_FONT))
                    .text_color(tokens::meta),
                ),
        )
        // A file too large for the tool to diff has its counts alone.
        .child(when(
            move || change.with(|change| !change.patch.is_empty()),
            move || {
                view().w_full().flex_col().child(each(
                    move || items.get(),
                    |item| match item {
                        Item::Line { at, .. } | Item::Gap { at, .. } => *at,
                    },
                    move |item, _| match item.with_untracked(|item| item.clone()) {
                        Item::Line {
                            sign,
                            text,
                            changed,
                            tokens,
                            ..
                        } => line_row(sign, text, changed, tokens).into_element(),
                        Item::Gap { lines, .. } => gap_row(lines).into_element(),
                    },
                ))
            },
        ))
}

/// One line of the change: its sign, then its text, the words that changed in
/// it set on the fill their side takes.
fn line_row(
    sign: char,
    body: SharedString,
    changed: Option<(usize, usize)>,
    tokens: Vec<(usize, usize, highlight::Token)>,
) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .px_2()
        .bg(move || match sign {
            '+' => tokens::diff_added(),
            '-' => tokens::diff_removed(),
            _ => AdaptiveColor::TRANSPARENT,
        })
        .child(
            text(SharedString::from(sign.to_string()))
                .flex_none()
                .font_family(code::MONO_FONT)
                .text_size(px(sizes::CODE_FONT))
                .line_height(LineHeight::Multiple(MESSAGE_LINE))
                .text_color(tokens::meta),
        )
        .child(segments(sign, body, changed, tokens))
}

/// The line's text, cut where a token or the changed words begin, each piece in
/// its token's ink and the words that changed on a deeper fill of its side.
fn segments(
    sign: char,
    value: SharedString,
    changed: Option<(usize, usize)>,
    tokens: Vec<(usize, usize, highlight::Token)>,
) -> Text {
    let len = value.len();
    let changed = changed.filter(|(start, end)| start < end && *end <= len);
    let inks: Vec<(usize, usize, highlight::Token)> = tokens
        .into_iter()
        .filter(|(start, end, _)| start < end && *end <= len)
        .collect();

    let mut cuts = vec![0, len];
    if let Some((start, end)) = changed {
        cuts.extend([start, end]);
    }
    for (start, end, _) in &inks {
        cuts.extend([*start, *end]);
    }
    cuts.retain(|at| value.is_char_boundary(*at));
    cuts.sort_unstable();
    cuts.dedup();

    let mut line = text("")
        .font_family(code::MONO_FONT)
        .text_size(px(sizes::CODE_FONT))
        .line_height(LineHeight::Multiple(MESSAGE_LINE))
        .text_color(move || theme().foreground);

    for cut in cuts.windows(2) {
        let (start, end) = (cut[0], cut[1]);
        if start >= end {
            continue;
        }
        let Some(piece) = value.get(start..end) else {
            continue;
        };

        let ink = inks
            .iter()
            .find(|(from, to, _)| *from <= start && end <= *to)
            .map(|(_, _, ink)| *ink);

        let mut part = text(SharedString::from(piece.to_owned()))
            .text_color(ink.map_or_else(|| theme().foreground, highlight::Token::color));
        if ink.is_some_and(highlight::Token::italic) {
            part = part.italic();
        }
        if let Some((from, to)) = changed
            && from <= start
            && end <= to
        {
            part = part.bg(move || match sign {
                '+' => tokens::diff_added_word(),
                '-' => tokens::diff_removed_word(),
                _ => AdaptiveColor::TRANSPARENT,
            });
        }

        line = line.child(part);
    }

    line
}

/// The lines a hunk leaves out: how many, and nothing of what they said.
fn gap_row(lines: usize) -> impl IntoElement {
    view()
        .w_full()
        .justify_center()
        .px_2()
        .py_1()
        .border_t_1()
        .border_b_1()
        .border_color(move || theme().border)
        .child(
            text(SharedString::from(format!("⋯ {lines} lines")))
                .font_family(code::MONO_FONT)
                .text_size(px(sizes::CODE_FONT))
                .text_color(tokens::meta),
        )
}

/// A hunk of a patch: where it sits in the file before the change, and the
/// lines it holds.
struct Hunk {
    old_start: usize,
    old_count: usize,
    lines: Vec<(char, String)>,
}

/// The card's rows for `patch`, its hunks' lines in the language `path` names,
/// each gap between them counted on a row of its own.
fn items_of(path: &str, patch: &str) -> Vec<Item> {
    let mut items = Vec::new();
    let mut at = 0;
    let mut previous_end: Option<usize> = None;
    for hunk in hunks_of(patch) {
        if let Some(end) = previous_end {
            let lines = hunk.old_start.saturating_sub(end);
            if lines > 0 {
                items.push(Item::Gap { at, lines });
                at += 1;
            }
        }
        previous_end = Some(hunk.old_start + hunk.old_count);

        let changed = changed_parts(&hunk.lines);
        let bodies: Vec<&str> = hunk.lines.iter().map(|(_, text)| text.as_str()).collect();
        let inks = highlight::spans(path, &bodies);
        for (index, ((sign, text), tokens)) in hunk.lines.into_iter().zip(inks).enumerate() {
            items.push(Item::Line {
                at,
                sign,
                text: text.into(),
                changed: changed[index],
                tokens,
            });
            at += 1;
        }
    }

    items
}

/// The hunks of `patch`, in order, without its file headers.
fn hunks_of(patch: &str) -> Vec<Hunk> {
    let mut hunks: Vec<Hunk> = Vec::new();
    for line in patch.lines() {
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            continue;
        }
        if let Some(rest) = line.strip_prefix("@@ -") {
            if let Some(hunk) = hunk_header(rest) {
                hunks.push(hunk);
            }
            continue;
        }
        let Some((sign, text)) = line.split_at_checked(1) else {
            continue;
        };
        let Some(sign) = sign.chars().next() else {
            continue;
        };
        if !matches!(sign, ' ' | '-' | '+') {
            continue;
        }
        if let Some(hunk) = hunks.last_mut() {
            hunk.lines.push((sign, text.to_owned()));
        }
    }

    hunks
}

/// The hunk a header's rest names (`12,3 +12,4 @@`, or `12 +12 @@` for one
/// line), with no lines yet.
fn hunk_header(rest: &str) -> Option<Hunk> {
    let (old, _) = rest.split_once(' ')?;
    let (start, count) = old.split_once(',').unwrap_or((old, "1"));

    Some(Hunk {
        old_start: start.parse().ok()?,
        old_count: count.parse().ok()?,
        lines: Vec::new(),
    })
}

/// Where each line's text changes, for the lines a removal and an addition pair
/// up: the lines they share at either end are what did not change.
fn changed_parts(lines: &[(char, String)]) -> Vec<Option<(usize, usize)>> {
    let mut parts = vec![None; lines.len()];
    let mut index = 0;
    while index < lines.len() {
        if lines[index].0 != '-' {
            index += 1;
            continue;
        }

        let removed_start = index;
        while index < lines.len() && lines[index].0 == '-' {
            index += 1;
        }
        let removed_end = index;
        let added_start = index;
        while index < lines.len() && lines[index].0 == '+' {
            index += 1;
        }
        let added_end = index;

        let paired = (removed_end - removed_start).min(added_end - added_start);
        for pair in 0..paired {
            let removed = &lines[removed_start + pair].1;
            let added = &lines[added_start + pair].1;
            let head = common_prefix(removed, added);
            let tail = common_suffix(removed, added, head);
            parts[removed_start + pair] = range(head, removed.len() - tail);
            parts[added_start + pair] = range(head, added.len() - tail);
        }
    }

    parts
}

/// The bytes `a` and `b` share at their start.
fn common_prefix(a: &str, b: &str) -> usize {
    a.char_indices()
        .zip(b.char_indices())
        .find(|((_, left), (_, right))| left != right)
        .map_or_else(|| a.len().min(b.len()), |(at, _)| at.0)
}

/// The bytes `a` and `b` share at their end, past the `head` they share.
fn common_suffix(a: &str, b: &str, head: usize) -> usize {
    let mut shared = 0;
    for (left, right) in a[head..].chars().rev().zip(b[head..].chars().rev()) {
        if left != right {
            break;
        }
        shared += left.len_utf8();
    }

    shared
}

/// `start..end`, or nothing when the line did not change there.
fn range(start: usize, end: usize) -> Option<(usize, usize)> {
    (start < end).then_some((start, end))
}
