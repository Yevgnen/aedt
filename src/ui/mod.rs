//! The app's styled layer: every control here takes a variant, size or
//! state, never a color or radius from the feature that places it.

pub(crate) mod avatar;
pub(crate) mod badge;
pub(crate) mod button;
pub(crate) mod collapsible;
pub(crate) mod empty;
pub(crate) mod field;
pub(crate) mod icon;
pub(crate) mod input;
pub(crate) mod list_item;
pub(crate) mod menu;
pub(crate) mod motion;
pub(crate) mod separator;
pub(crate) mod sizes;
pub(crate) mod switch;
pub(crate) mod tokens;

/// A message's line height, a multiple of its font size: the prose and the
/// code blocks it holds read at the same density.
pub(crate) const MESSAGE_LINE: f32 = 1.25;
