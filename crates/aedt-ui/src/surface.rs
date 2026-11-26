//! The cards that float above the app: a menu's list, a message's preview, the
//! composer, the settings dialog. They share one recipe — the card's fill, a
//! hairline edge and the app's floating shadow — and differ only in how round
//! the corner is and how strong the edge reads.

use nsh_base::theme;
use nsh_core::{element::StyleRefinement, prelude::*};

use crate::tokens;

/// Which card floats here.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// A menu or a picker's list, in the app's own radius.
    Popup,
    /// A message's preview over the rail's ticks, in the same radius.
    Preview,
    /// The composer: a wider corner and a lighter edge, so it reads as an input
    /// laid on the conversation rather than as a card.
    Composer,
    /// The settings dialog.
    Dialog,
}

impl Surface {
    /// The card's corner.
    fn corner(self) -> impl Fn() -> Length + Copy {
        move || match self {
            Self::Popup | Self::Preview => Length::from(theme().radius),
            Self::Composer => Length::from(px(24.0)),
            Self::Dialog => Length::from(px(16.0)),
        }
    }

    /// The hairline around it: the theme's border, a shade lighter on the
    /// composer.
    fn edge(self) -> impl Fn() -> AdaptiveColor + Copy {
        move || match self {
            Self::Composer => theme().border.opacity(0.6),
            _ => theme().border,
        }
    }
}

/// `style` with the card's fill, edge, corner and shadow: what a popup's own
/// style starts from.
pub fn restyle(surface: Surface, style: StyleRefinement) -> StyleRefinement {
    style
        .rounded(surface.corner())
        .border_1()
        .border_color(surface.edge())
        .bg(move || theme().card)
        .shadow(tokens::floating_shadow())
}

/// A card of `surface`, for the places that place one as an element.
pub fn card(surface: Surface) -> View {
    view()
        .rounded(surface.corner())
        .border_1()
        .border_color(surface.edge())
        .bg(move || theme().card)
        .shadow(tokens::floating_shadow())
}
