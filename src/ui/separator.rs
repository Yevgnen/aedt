//! A hairline between regions.

use nsh_base::theme;
use nsh_core::prelude::*;

/// The one-line separator two adjacent regions share.
pub(crate) fn horizontal() -> impl IntoElement {
    view().w_full().h(px(1.0)).bg(move || theme().border)
}
