//! The app's own theme roles: the tokens its look adds to shadcn's set, each
//! read here so a page names the role instead of a string and a fallback.

use nsh_base::theme;
use nsh_core::{Color, color::black, prelude::*};

/// A user message's bubble.
pub fn bubble() -> AdaptiveColor {
    theme().color("bubble").unwrap_or_else(|| theme().primary)
}

/// A code block's fill, the token `code` (the theme's `muted` when it leaves
/// that out). It is lighter than `muted`, so a block reads as a soft inset
/// rather than a gray box, and its border takes the same color.
pub fn code() -> AdaptiveColor {
    theme().color("code").unwrap_or_else(|| theme().muted)
}

/// What a control that is on is filled with, a switch's track: the theme's
/// `checked`, or its `ring` blue when the theme leaves that out. It is stronger
/// than `primary`, which this theme keeps pale for filled buttons.
pub fn checked() -> AdaptiveColor {
    theme().color("checked").unwrap_or_else(|| theme().ring)
}

/// A connected or healthy state: the theme's `success`, or the checked blue
/// when the theme leaves that out.
pub fn success() -> AdaptiveColor {
    theme().color("success").unwrap_or_else(checked)
}

/// The quietest ink the app has: the theme's `meta`, or the muted one when the
/// theme leaves it out. Thinking and the rail's answer read in it.
pub fn meta() -> AdaptiveColor {
    theme()
        .color("meta")
        .unwrap_or_else(|| theme().muted_foreground)
}

/// A message's metadata line: the theme's `caption`, or the muted one when the
/// theme leaves it out.
pub fn caption() -> AdaptiveColor {
    theme()
        .color("caption")
        .unwrap_or_else(|| theme().muted_foreground)
}

/// An added line's fill in a diff: the theme's `diff-added`, or the code
/// block's own when the theme leaves that out, so a diff with no colors of its
/// own reads as the block it sits in.
pub fn diff_added() -> AdaptiveColor {
    theme().color("diff-added").unwrap_or_else(code)
}

/// The words changed within an added line: the theme's `diff-added-word`, or
/// the added line's fill when the theme leaves that out.
pub fn diff_added_word() -> AdaptiveColor {
    theme().color("diff-added-word").unwrap_or_else(diff_added)
}

/// A deleted line's fill in a diff: the theme's `diff-removed`, or the code
/// block's own when the theme leaves that out.
pub fn diff_removed() -> AdaptiveColor {
    theme().color("diff-removed").unwrap_or_else(code)
}

/// The words changed within a deleted line: the theme's `diff-removed-word`,
/// or the deleted line's fill when the theme leaves that out.
pub fn diff_removed_word() -> AdaptiveColor {
    theme()
        .color("diff-removed-word")
        .unwrap_or_else(diff_removed)
}

/// A floating card's shadow: two wide, faint layers, so the card seems to rest
/// on haze instead of sitting inside a dark edge. A theme file carries colors,
/// not shadows, so the app's one shadow is a token here with the rest.
pub fn floating_shadow() -> Vec<BoxShadow> {
    let layer = |offset_y: f32, blur: f32, alpha: u8| BoxShadow {
        offset_x: Points(0.0),
        offset_y: Points(offset_y),
        blur: Points(blur),
        spread: Points(0.0),
        color: Color::rgba(0, 0, 0, alpha),
        inset: false,
    };
    vec![layer(1.0, 2.0, 10), layer(6.0, 20.0, 13)]
}

/// The dim a modal lays over the window: black, faint enough that the app
/// behind it still reads as there. A theme file names roles, not a modal's
/// scrim, so the app's one scrim is a token here with the rest.
pub fn scrim() -> Hsla {
    black().opacity(0.28)
}
