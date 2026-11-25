//! Control and pane sizes, in points, shared by the app's components.

/// The sidebar's comfortable width: wide enough for session titles, visibly
/// subordinate to the conversation.
pub(crate) const SIDEBAR: f32 = 264.0;

/// A control's height.
pub(crate) const CONTROL: f32 = 32.0;

/// A list row's smallest height.
pub(crate) const LIST_ROW: f32 = 36.0;

/// An avatar's diameter.
pub(crate) const AVATAR: f32 = 28.0;

/// The composer's smallest height, before it grows with its text.
pub(crate) const COMPOSER_MIN: f32 = 76.0;

/// A user message's widest line, so a bubble does not become a page.
pub(crate) const MESSAGE_MAX: f32 = 560.0;
