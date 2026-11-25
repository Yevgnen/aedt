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

/// Three dots in a row (Lucide's `ellipsis`): a menu that holds more than its
/// trigger names.
const MORE: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>"#,
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

/// A plus (Lucide's `plus`): an action that adds something.
const PLUS: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<path d="M5 12h14"/>"#,
    r#"<path d="M12 5v14"/>"#,
    r#"</svg>"#,
);

/// The menu mark, `size` points square, in the caller's text color.
pub fn more(size: f32) -> impl IntoElement {
    icon(MORE, size)
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

/// The plus, `size` points square, in the caller's text color.
pub fn plus(size: f32) -> impl IntoElement {
    icon(PLUS, size)
}

/// `markup` drawn at `size` points square.
fn icon(markup: &str, size: f32) -> impl IntoElement {
    svg(ImageSrc::svg_markup(markup)).size(px(size))
}
