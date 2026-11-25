//! The app's own theme roles: the tokens its look adds to shadcn's set, each
//! read here so a page names the role instead of a string and a fallback.

use nsh_base::theme;
use nsh_core::prelude::*;

/// A user message's bubble.
pub(crate) fn bubble() -> AdaptiveColor {
    theme().color("bubble").unwrap_or_else(|| theme().primary)
}

/// A code block's fill, the token `code` (the theme's `muted` when it leaves
/// that out). It is lighter than `muted`, so a block reads as a soft inset
/// rather than a gray box, and its border takes the same color.
pub(crate) fn code() -> AdaptiveColor {
    theme().color("code").unwrap_or_else(|| theme().muted)
}

/// The quietest ink the app has: the theme's `meta`, or the muted one when the
/// theme leaves it out. Thinking and the rail's answer read in it.
pub(crate) fn meta() -> AdaptiveColor {
    theme()
        .color("meta")
        .unwrap_or_else(|| theme().muted_foreground)
}

/// A message's metadata line: the theme's `caption`, or the muted one when the
/// theme leaves it out.
pub(crate) fn caption() -> AdaptiveColor {
    theme()
        .color("caption")
        .unwrap_or_else(|| theme().muted_foreground)
}
