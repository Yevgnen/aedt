//! The app's icons: inline SVG, stroked in the text color the caller sets.

use nsh_base::theme;
use nsh_core::{element::ImageSrc, prelude::*};

/// Two rounded squares, the front one clear of the back: the copy mark every
/// editor draws (Lucide's `copy`), at its 24-unit geometry.
const COPY: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none""#,
    r#" stroke="currentColor" stroke-width="2" stroke-linecap="round""#,
    r#" stroke-linejoin="round">"#,
    r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/>"#,
    r#"<path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#,
    r#"</svg>"#,
);

/// The copy mark, `size` points square, in the muted ink.
pub(crate) fn copy(size: f32) -> impl IntoElement {
    svg(ImageSrc::svg_markup(COPY))
        .size(px(size))
        .text_color(move || theme().muted_foreground)
}
