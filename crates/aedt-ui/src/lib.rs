//! The app's styled layer: every control here takes a variant, size or
//! state, never a color or radius from the feature that places it.

pub mod attachment;
pub mod avatar;
pub mod badge;
pub mod button;
pub mod code;
pub mod collapsible;
pub mod diff;
pub mod empty;
pub mod field;
pub mod highlight;
pub mod icon;
pub mod input;
pub mod list_item;
pub mod markdown;
pub mod menu;
pub mod motion;
pub mod separator;
pub mod sizes;
pub mod state;
pub mod surface;
pub mod switch;
pub mod tokens;

/// A message's line height, a multiple of its font size: the prose and the
/// code blocks it holds read at the same density.
pub const MESSAGE_LINE: f32 = 1.25;
