//! Control and pane sizes, in points, shared by the app's components.

/// The sidebar's comfortable width: wide enough for session titles, visibly
/// subordinate to the conversation.
pub const SIDEBAR: f32 = 264.0;

/// The room the conversation keeps beside a panel that shows with it: its
/// column with the composer's row whole. A window narrower than this, once the
/// panels are counted, steps them aside — the files panel first, which counts
/// the sidebar, and the sidebar once not even one of them fits.
pub const ROOM: f32 = 490.0;

/// The sidebar's content inset: the line the traffic lights stand on, so a
/// heading, a group's mark and a session's title start on it too.
pub const SIDEBAR_INSET: f32 = 16.0;

/// The app's own top row under the hidden title bar: the sidebar's brand and
/// the conversation's header are this tall, and the traffic lights are centered
/// in it, so the three share one line.
pub const TOPBAR: f32 = 44.0;

/// The sidebar's heading row under the top row: its title and the action that
/// adds a workspace. The conversation's header takes the same height under its
/// own top row, so the title there reads on the line between the two.
pub const HEADING: f32 = 28.0;

/// A control's height.
pub const CONTROL: f32 = 32.0;

/// A list row's smallest height.
pub const LIST_ROW: f32 = 28.0;

/// A list row's side padding: less than the list's own inset, so a title sits
/// close to the edge of its row.
pub const LIST_ROW_INSET: f32 = 10.0;

/// A small square control's corner: half the app's radius, which on a 20-point
/// button keeps it a rounded square rather than the circle the full radius
/// would make of it.
pub const ICON_RADIUS: f32 = 6.0;

/// An avatar's diameter.
pub const AVATAR: f32 = 28.0;

/// The composer's smallest height, before it grows with its text.
pub const COMPOSER_MIN: f32 = 44.0;

/// The card's inner padding: the message text starts on this same inset from
/// the column's sides.
pub const CARD_PADDING: f32 = 12.0;

/// The round send button's diameter.
pub const SEND: f32 = 28.0;

/// A user message's widest line, so a bubble does not become a page.
pub const MESSAGE_MAX: f32 = 560.0;

/// A meta row's trailing icon button: small enough to sit beside 12pt text.
pub const META_ACTION: f32 = 20.0;

/// An attachment chip's height in the composer: shorter than a control, on the
/// line a 12pt label keeps.
pub const CHIP: f32 = 24.0;

/// An attachment chip's widest label, so one long file name does not push the
/// composer's row.
pub const CHIP_MAX_W: f32 = 220.0;

/// The message rail at the conversation's right edge.
pub const RAIL: f32 = 32.0;

/// The rail's inset from the window's edge: the ticks do not touch it.
pub const RAIL_INSET: f32 = 14.0;

/// A row's secondary text: a point under a control's 14pt label, and the size
/// an inline code keeps, so a value beside a title reads quieter than the title
/// without dropping to a caption's 12 points.
pub const TEXT_COMPACT: f32 = 13.0;

/// A code block's font size: two points under the 13pt an inline code keeps,
/// so a block reads as a quotation of code rather than as louder body text.
pub const CODE_FONT: f32 = 11.0;

/// The window the app opens in, in points.
pub const WINDOW_W: f32 = 1320.0;
pub const WINDOW_H: f32 = 880.0;

/// The least height the window may take: the header and the composer card with
/// its status line keep their room, and a few exchanges still read above the
/// composer.
pub const WINDOW_MIN_H: f32 = 560.0;

/// The least width the window may take: the rail's 32 points on the
/// conversation's two sides, and the column the settings' default blank leaves
/// — enough for the composer's row, the model's name beside its effort, its
/// chevron and the send button, to show whole. The sidebar steps aside well
/// above it ([`NARROW`]), so the floor is the conversation alone.
pub const WINDOW_MIN_W: f32 = 520.0;

/// The card a message's question opens in, over the rail's ticks.
pub const PREVIEW_W: f32 = 320.0;

/// The picker's list: wide enough for a model's name beside its provider, and
/// no taller than a few of them before it scrolls.
pub const PICKER_MIN_W: f32 = 240.0;
pub const PICKER_MAX_H: f32 = 360.0;

/// The picker's effort list: one short word a row.
pub const EFFORT_W: f32 = 160.0;

/// The settings dialog, and the column its categories take inside it.
pub const DIALOG_W: f32 = 720.0;
pub const DIALOG_H: f32 = 520.0;
pub const CATEGORIES_W: f32 = 180.0;
