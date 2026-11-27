//! The app's icons: inline SVG, stroked in the text color the caller sets.

use nsh_core::{element::ImageSrc, prelude::*};

/// The two rounded squares of the copy mark (Lucide's `copy`): the front one
/// clear of the back, at its 24-unit geometry.
const COPY: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/>"#,
    r#"<path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#,
    r#"</svg>"#,
);

/// A bulb over two lines of base (Lucide's `lightbulb`).
const BULB: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/>"#,
    r#"<path d="M9 18h6"/>"#,
    r#"<path d="M10 22h4"/>"#,
    r#"</svg>"#,
);

/// A microphone (Lucide's `mic`): what the reader says.
const MIC: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/>"#,
    r#"<path d="M19 10v2a7 7 0 0 1-14 0v-2"/>"#,
    r#"<path d="M12 19v3"/>"#,
    r#"</svg>"#,
);

/// The copy mark, `size` points square. It is stroked in `currentColor`, so the
/// color is the caller's `text_color`.
pub fn copy(size: f32) -> impl IntoElement {
    icon(COPY, size)
}

/// The bulb beside a thought the reader can open, `size` points square, in the
/// caller's text color.
pub fn bulb(size: f32) -> impl IntoElement {
    icon(BULB, size)
}

/// The microphone, `size` points square, in the caller's text color.
pub fn mic(size: f32) -> impl IntoElement {
    icon(MIC, size)
}

/// Three dots in a row (Lucide's `ellipsis`): a menu that holds more than its
/// trigger names.
const MORE: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>"#,
    r#"</svg>"#,
);

/// A box with its lid (Lucide's `archive`): puts a conversation away.
const ARCHIVE: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<rect width="20" height="5" x="2" y="3" rx="1"/>"#,
    r#"<path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8"/>"#,
    r#"<path d="M10 12h4"/>"#,
    r#"</svg>"#,
);

/// A bin with its lid (Lucide's `trash-2`): takes a conversation out for
/// good.
const TRASH: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M3 6h18"/>"#,
    r#"<path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>"#,
    r#"<path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>"#,
    r#"<path d="M10 11v6"/><path d="M14 11v6"/>"#,
    r#"</svg>"#,
);

/// A gear (Lucide's `settings`).
const SETTINGS: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"/>"#,
    r#"<circle cx="12" cy="12" r="3"/>"#,
    r#"</svg>"#,
);

/// A mark turning back on itself (Lucide's `rotate-ccw`): puts a value back as
/// it was.
const ROTATE_CCW: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>"#,
    r#"<path d="M3 3v5h5"/>"#,
    r#"</svg>"#,
);

/// A refresh mark (Lucide's `refresh-cw`): asks a list to load again.
const REFRESH: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>"#,
    r#"<path d="M21 3v5h-5"/>"#,
    r#"<path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>"#,
    r#"<path d="M8 16H3v5"/>"#,
    r#"</svg>"#,
);

/// A database (Lucide's `database`): the models a provider lists.
const DATABASE: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<ellipse cx="12" cy="5" rx="9" ry="3"/>"#,
    r#"<path d="M3 5V19A9 3 0 0 0 21 19V5"/>"#,
    r#"<path d="M3 12A9 3 0 0 0 21 12"/>"#,
    r#"</svg>"#,
);

/// A check (Lucide's `check`): the one a picker has chosen.
const CHECK: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M20 6 9 17l-5-5"/>"#,
    r#"</svg>"#,
);

/// Two checks (Lucide's `check-check`): every row of a list is on.
const CHECKS: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M18 6 7 17l-5-5"/>"#,
    r#"<path d="m22 10-7.5 7.5L13 16"/>"#,
    r#"</svg>"#,
);

/// A chevron pointing down (Lucide's `chevron-down`): the picker opens from
/// the row it names.
const CHEVRON_DOWN: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="m6 9 6 6 6-6"/>"#,
    r#"</svg>"#,
);

/// A chevron pointing right (Lucide's `chevron-right`): a menu row whose
/// items open beside it.
const CHEVRON_RIGHT: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="m9 18 6-6-6-6"/>"#,
    r#"</svg>"#,
);

/// A chevron pointing up (Lucide's `chevron-up`): what is folded opens, and
/// an open fold closes with one pointing down.
const CHEVRON_UP: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="m18 15-6-6-6 6"/>"#,
    r#"</svg>"#,
);

/// A wrench (Lucide's `wrench`): a tool the reply ran.
const WRENCH: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    // A stroke under the set's: the wrench's long handle and its ring read
    // heavier than the other marks at the size a row sets them in.
    r#" stroke="currentColor" stroke-width="1.5" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>"#,
    r#"</svg>"#,
);

/// A plug (Lucide's `plug`): the connection to a local MCP server.
const PLUG: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M12 22v-5"/>"#,
    r#"<path d="M9 8V2"/><path d="M15 8V2"/>"#,
    r#"<path d="M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z"/>"#,
    r#"</svg>"#,
);

/// A folder with a plus (Lucide's `folder-plus`): adds a workspace.
const FOLDER_PLUS: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>"#,
    r#"<path d="M12 10v6"/><path d="M9 13h6"/>"#,
    r#"</svg>"#,
);

/// A folder with a minus (Lucide's `folder-minus`): a workspace whose
/// conversations are folded away.
const FOLDER_MINUS: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>"#,
    r#"<path d="M9 13h6"/>"#,
    r#"</svg>"#,
);

/// An open folder (Lucide's `folder-open`): the workspace whose
/// conversations follow.
const FOLDER_OPEN: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/>"#,
    r#"</svg>"#,
);

/// A frame with its left column marked (Lucide's `panel-left`): the sidebar a
/// press shows or hides.
const PANEL_LEFT: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<rect width="18" height="18" x="3" y="3" rx="2"/>"#,
    r#"<path d="M9 3v18"/>"#,
    r#"</svg>"#,
);

/// A frame with its right column marked (Lucide's `panel-right`): the files a
/// press shows or hides.
const PANEL_RIGHT: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<rect width="18" height="18" x="3" y="3" rx="2"/>"#,
    r#"<path d="M15 3v18"/>"#,
    r#"</svg>"#,
);

/// Zed's logomark (Simple Icons' `zedindustries`): the editor a conversation's
/// workspace opens in. The mark is filled rather than stroked, as the brand
/// draws it, so it holds its shape at the size a header sets its marks in.
const ZED: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">"#,
    r#"<path fill="currentColor" d="M2.25 1.5a.75.75 0 0 0-.75.75v16.5H0V2.25A2.25 2.25 0 0 1 2.25 0h20.095c1.002 0 1.504 1.212.795 1.92L10.764 14.298h3.486V12.75h1.5v1.922a1.125 1.125 0 0 1-1.125 1.125H9.264l-2.578 2.578h11.689V9h1.5v9.375a1.5 1.5 0 0 1-1.5 1.5H5.185L2.562 22.5H21.75a.75.75 0 0 0 .75-.75V5.25H24v16.5A2.25 2.25 0 0 1 21.75 24H1.655C.653 24 .151 22.788.86 22.08L13.19 9.75H9.75v1.5h-1.5V9.375A1.125 1.125 0 0 1 9.375 8.25h5.314l2.625-2.625H5.625V15h-1.5V5.625a1.5 1.5 0 0 1 1.5-1.5h13.19L21.438 1.5z"/>"#,
    r#"</svg>"#,
);

/// A folder (Lucide's `folder`): a directory the files panel lists closed.
const FOLDER: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>"#,
    r#"</svg>"#,
);

/// A page with its corner folded (Lucide's `file`): a file the panel lists.
const FILE: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/>"#,
    r#"<path d="M14 2v4a2 2 0 0 0 2 2h4"/>"#,
    r#"</svg>"#,
);

/// A plus (Lucide's `plus`): an action that adds something.
const PLUS: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M5 12h14"/>"#,
    r#"<path d="M12 5v14"/>"#,
    r#"</svg>"#,
);

/// Two strokes crossing (Lucide's `x`): takes an attachment out of a message.
const X: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
    r#"</svg>"#,
);

/// The menu mark, `size` points square, in the caller's text color.
pub fn more(size: f32) -> impl IntoElement {
    icon(MORE, size)
}

/// The archive mark, `size` points square, in the caller's text color.
pub fn archive(size: f32) -> impl IntoElement {
    icon(ARCHIVE, size)
}

/// The bin, `size` points square, in the caller's text color.
pub fn trash(size: f32) -> impl IntoElement {
    icon(TRASH, size)
}

/// The settings mark, `size` points square, in the caller's text color.
pub fn settings(size: f32) -> impl IntoElement {
    icon(SETTINGS, size)
}

/// The refresh mark, `size` points square, in the caller's text color.
pub fn refresh(size: f32) -> impl IntoElement {
    icon(REFRESH, size)
}

/// The mark a value resets by, `size` points square, in the caller's text
/// color.
pub fn rotate_ccw(size: f32) -> impl IntoElement {
    icon(ROTATE_CCW, size)
}

/// The models mark, `size` points square, in the caller's text color.
pub fn database(size: f32) -> impl IntoElement {
    icon(DATABASE, size)
}

/// The check mark, `size` points square, in the caller's text color.
pub fn check(size: f32) -> impl IntoElement {
    icon(CHECK, size)
}

/// The double check mark, `size` points square, in the caller's text color:
/// every row of a list on.
pub fn checks(size: f32) -> impl IntoElement {
    icon(CHECKS, size)
}

/// The chevron pointing down, `size` points square, in the caller's text
/// color.
pub fn chevron_down(size: f32) -> impl IntoElement {
    icon(CHEVRON_DOWN, size)
}

/// The chevron pointing right, `size` points square, in the caller's text
/// color.
pub fn chevron_right(size: f32) -> impl IntoElement {
    icon(CHEVRON_RIGHT, size)
}

/// The chevron pointing up, `size` points square, in the caller's text color.
pub fn chevron_up(size: f32) -> impl IntoElement {
    icon(CHEVRON_UP, size)
}

/// The wrench, `size` points square, in the caller's text color.
pub fn wrench(size: f32) -> impl IntoElement {
    icon(WRENCH, size)
}

/// The plug, `size` points square, in the caller's text color.
pub fn plug(size: f32) -> impl IntoElement {
    icon(PLUG, size)
}

/// The folder with a plus, `size` points square, in the caller's text color.
pub fn folder_plus(size: f32) -> impl IntoElement {
    icon(FOLDER_PLUS, size)
}

/// The files mark, `size` points square, in the caller's text color.
pub fn panel_right(size: f32) -> impl IntoElement {
    icon(PANEL_RIGHT, size)
}

/// The closed folder, `size` points square, in the caller's text color.
pub fn folder(size: f32) -> impl IntoElement {
    icon(FOLDER, size)
}

/// The page with its corner folded, `size` points square, in the caller's text
/// color.
pub fn file(size: f32) -> impl IntoElement {
    icon(FILE, size)
}

/// The open folder, `size` points square, in the caller's text color.
pub fn folder_open(size: f32) -> impl IntoElement {
    icon(FOLDER_OPEN, size)
}

/// The folder with a minus, `size` points square, in the caller's text color.
pub fn folder_minus(size: f32) -> impl IntoElement {
    icon(FOLDER_MINUS, size)
}

/// The sidebar mark, `size` points square, in the caller's text color.
pub fn panel_left(size: f32) -> impl IntoElement {
    icon(PANEL_LEFT, size)
}

/// Zed's logomark, `size` points square, in the caller's text color.
pub fn zed(size: f32) -> impl IntoElement {
    icon(ZED, size)
}

/// The plus, `size` points square, in the caller's text color.
pub fn plus(size: f32) -> impl IntoElement {
    icon(PLUS, size)
}

/// The cross, `size` points square, in the caller's text color.
pub fn x(size: f32) -> impl IntoElement {
    icon(X, size)
}

/// `markup` drawn at `size` points square.
fn icon(markup: &str, size: f32) -> impl IntoElement {
    svg(ImageSrc::svg_markup(markup)).size(px(size))
}
