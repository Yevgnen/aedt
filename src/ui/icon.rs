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
pub(crate) fn copy(size: f32) -> impl IntoElement {
    icon(COPY, size)
}

/// The bulb beside a thought the reader can open, `size` points square, in the
/// caller's text color.
pub(crate) fn bulb(size: f32) -> impl IntoElement {
    icon(BULB, size)
}

/// `markup` drawn at `size` points square.
fn icon(markup: &str, size: f32) -> impl IntoElement {
    svg(ImageSrc::svg_markup(markup)).size(px(size))
}
