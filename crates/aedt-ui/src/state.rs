//! The states every control shares: the same ring while the keyboard is on it,
//! and the same dim while it cannot be used.

use nsh_base::theme;
use nsh_core::{element::StyleRefinement, prelude::*};

/// The ring a control wears while the keyboard is on it: two points of the
/// theme's ring color, drawn outside its box.
pub fn focus_ring(style: StyleRefinement) -> StyleRefinement {
    style.outline(px(2.0)).outline_color(move || theme().ring)
}
