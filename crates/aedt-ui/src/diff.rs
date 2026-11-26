//! A file a call changed, drawn as a card: the file's name and how far the call
//! changed it, over the rows of its diff with the words changed within them.

use nsh_base::{
    DiffIndicators, DiffInline, DiffLayout, DiffViewStyles, diff_view, text_view::PartStyle, theme,
};
use nsh_core::{PropertyValue, StyleKey, element::Style, prelude::*};

use crate::{code, sizes, tokens};

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

/// How tall one row of a diff is, in points.
const ROW: f32 = 14.0;

/// How many rows a diff shows before it scrolls its own: a change longer than
/// this is read through the view rather than pushing the reply down.
const ROWS: f32 = 24.0;

/// The card of one file a call changed.
pub fn card(change: impl Fn() -> Change + Copy + 'static) -> View {
    let change = computed(change);

    view()
        .w_full()
        .rounded(move || theme().radius)
        // The rows' fills are square: the card's corners keep them.
        .overflow_hidden()
        .bg(tokens::code)
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap_2()
                // The bar a diff view puts over a change's first row; the file's
                // name takes its place.
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
            move || rows(change),
        ))
}

/// The rows of one file's change, drawn by the diff view: the rows it changed
/// and the words changed within them, in the theme's own fills.
///
/// The view is given the two texts rather than the patch: a patch carries a
/// header for each of its hunks, and the view draws one as a row of its own — a
/// band over the first change, which a card that names no lines has no use for.
/// The two texts hold the hunks' lines alone, so every row is a line of them.
///
/// The view lays its rows out absolutely and fills the box it is given, so the
/// card gives it the height they take — counted from the patch, its header
/// lines aside — up to [`ROWS`], past which the view scrolls them.
fn rows(change: Memo<Change>) -> impl IntoElement {
    let sides = computed(move || change.with(|change| sides_of(&change.patch)));
    let count = move || change.with(|change| rows_of(&change.patch));

    view()
        .w_full()
        .h(move || px(count().min(ROWS) * ROW))
        .child(
            diff_view()
                .old_value(move || sides.with(|sides| Some(sides.before.clone())))
                .new_value(move || sides.with(|sides| Some(sides.after.clone())))
                // The hunks' own lines are the whole of what reads: a line
                // kept around a change is in them already.
                .context_lines(0)
                .layout(DiffLayout::Unified)
                .inline(DiffInline::Word)
                // The view's own fills are its green and red; the card's
                // are the theme's, and no second one under them.
                .background(false)
                // The card names no lines: a change in a reply is read for what
                // it did, not for where in the file it sits.
                .line_numbers(false)
                // Its fills say what changed; a bar beside them says it again.
                .indicators(DiffIndicators::None)
                // Every row shows, so the height counted above is the
                // height the rows take.
                .expand_all(true)
                .font_size(px(sizes::CODE_FONT))
                .line_height(px(ROW))
                .styles(diff_styles)
                // The view fills the box the card measured for it.
                .size_full(),
        )
}

/// How many rows `patch` draws: its lines, less the file's two headers and its
/// hunks' own.
fn rows_of(patch: &str) -> f32 {
    patch
        .lines()
        .filter(|line| {
            !line.starts_with("--- ")
                && !line.starts_with("+++ ")
                && matches!(line.as_bytes().first(), Some(b' ' | b'-' | b'+'))
        })
        .count()
        .max(1) as f32
}

/// The two texts `patch` compares: what the file held and what it holds, each
/// of the hunks' lines in order, the untouched lines between hunks left out
/// with the headers that name them.
#[derive(PartialEq)]
struct Sides {
    before: SharedString,
    after: SharedString,
}

fn sides_of(patch: &str) -> Sides {
    let mut before = String::new();
    let mut after = String::new();
    for line in patch.lines() {
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            continue;
        }
        let Some((sign, text)) = line.split_at_checked(1) else {
            continue;
        };
        match sign {
            " " => {
                before.push_str(text);
                before.push('\n');
                after.push_str(text);
                after.push('\n');
            }
            "-" => {
                before.push_str(text);
                before.push('\n');
            }
            "+" => {
                after.push_str(text);
                after.push('\n');
            }
            // A hunk's header, and the note a last line without its own ending
            // leaves: neither is a line of either text.
            _ => {}
        }
    }

    Sides {
        before: before.into(),
        after: after.into(),
    }
}

/// The diff's fills: the theme's own, over the view's green and red.
fn diff_styles() -> DiffViewStyles {
    let mut styles = DiffViewStyles::default();
    styles.line_added = Some(fill(tokens::diff_added()));
    styles.line_deleted = Some(fill(tokens::diff_removed()));
    styles.word_added = Some(fill(tokens::diff_added_word()));
    styles.word_deleted = Some(fill(tokens::diff_removed_word()));
    // A stretch of lines the view leaves out is not a row of this card: the
    // rows it shows are the whole of the change.
    styles.separator = Some(faded());

    styles
}

/// A part that takes its row but shows nothing of it.
fn faded() -> PartStyle {
    PartStyle(Style::default().with(StyleKey::Opacity, PropertyValue::Number(0.0.into())))
}

/// `color` as a part's fill, in the appearance now: the view's own fills are
/// styles on its elements, which carry both appearances, while a part's style
/// is a patch of declarations, which carries one color.
fn fill(color: AdaptiveColor) -> PartStyle {
    PartStyle(Style::default().with(
        StyleKey::BackgroundColor,
        PropertyValue::Color(Some(nsh_core::element::color(color).into())),
    ))
}
