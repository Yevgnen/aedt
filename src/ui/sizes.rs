//! Control and pane sizes, in points, shared by the app's components.

/// The sidebar's comfortable width: wide enough for session titles, visibly
/// subordinate to the conversation.
pub(crate) const SIDEBAR: f32 = 264.0;

/// A control's height.
pub(crate) const CONTROL: f32 = 32.0;

/// A list row's smallest height.
pub(crate) const LIST_ROW: f32 = 32.0;

/// A list row's side padding: less than the list's own inset, so a title sits
/// close to the edge of its row.
pub(crate) const LIST_ROW_INSET: f32 = 10.0;

/// An avatar's diameter.
pub(crate) const AVATAR: f32 = 28.0;

/// The composer's smallest height, before it grows with its text.
pub(crate) const COMPOSER_MIN: f32 = 44.0;

/// The conversation column's widest block: the message list and the composer
/// card share it, so they stay one column on wide windows.
pub(crate) const COMPOSER_MAX: f32 = 760.0;

/// The conversation column's side inset: the composer card sits inside it.
pub(crate) const COMPOSER_SIDE: f32 = 24.0;

/// The card's inner padding: the message text starts on this same inset from
/// the column's sides.
pub(crate) const CARD_PADDING: f32 = 12.0;

/// The round send button's diameter.
pub(crate) const SEND: f32 = 28.0;

/// A user message's widest line, so a bubble does not become a page.
pub(crate) const MESSAGE_MAX: f32 = 560.0;

/// A meta row's trailing icon button: small enough to sit beside 12pt text.
pub(crate) const META_ACTION: f32 = 20.0;

/// The message rail at the conversation's right edge.
pub(crate) const RAIL: f32 = 32.0;

/// The rail's inset from the window's edge: the ticks do not touch it.
pub(crate) const RAIL_INSET: f32 = 14.0;

/// A code block's font size: two points under the 13pt an inline code keeps,
/// so a block reads as a quotation of code rather than as louder body text.
pub(crate) const CODE_FONT: f32 = 11.0;
