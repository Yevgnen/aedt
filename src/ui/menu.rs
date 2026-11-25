//! The app's menus: the row that opens one, the card its items open in, and an
//! item of it. `nsh/base` owns the behavior (placement, focus, keyboard
//! navigation, dismissal); the look is here.

use nsh_base::{
    ButtonElement, MenuElement, MenuItemElement, button, menu as base_menu, menu_item as base_item,
    theme,
};
use nsh_core::prelude::*;

use crate::ui::{motion, sizes, tokens};

/// A menu that `trigger` opens, its `items` in the app's floating card. It
/// goes above the trigger, where a command at the window's edge has room.
pub(crate) fn menu(trigger: impl IntoElement, items: impl IntoElement) -> MenuElement {
    base_menu()
        .placement(Placement::Top)
        .offset(px(4.0))
        .popup_style(|s| {
            s.p_1()
                .rounded(move || theme().radius)
                .border_1()
                .border_color(move || theme().border)
                .bg(move || theme().card)
                .shadow(tokens::floating_shadow())
        })
        .trigger(trigger)
        .child(items)
}

/// A row that opens a menu: a mark and its label, in the app's quiet chrome.
/// The menu opens it on a press and with the keyboard; it takes no press of
/// its own.
pub(crate) fn trigger(
    mark: impl IntoElement,
    label: impl IntoReactive<SharedString>,
) -> ButtonElement {
    button()
        .w_full()
        .flex_row()
        .items_center()
        .gap_2()
        .min_h(px(sizes::LIST_ROW))
        .px(px(sizes::LIST_ROW_INSET))
        .rounded(move || theme().radius)
        .text_color(move || theme().muted_foreground)
        .hover(|s| s.bg(move || theme().muted))
        // The menu marks its trigger expanded while it is open, and the
        // slot's props land on this row, so both states are drawn here.
        .expanded(|s| s.bg(move || theme().accent))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(mark)
        .child(text(label).text_sm())
}

/// An item of one of those menus: a mark and its label, in the app's row look.
/// Choosing it closes the menu.
pub(crate) fn item(
    mark: impl IntoElement,
    label: impl IntoReactive<SharedString>,
) -> MenuItemElement {
    base_item()
        .flex_row()
        .items_center()
        .gap_2()
        .min_h(px(sizes::LIST_ROW))
        .px(px(sizes::LIST_ROW_INSET))
        .rounded(move || theme().radius)
        .text_color(move || theme().foreground)
        .hover(|s| s.bg(move || theme().muted))
        .focus(|s| s.bg(move || theme().muted))
        .active(|s| s.bg(move || theme().accent))
        .disabled_style(|s| s.opacity(0.4))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(mark)
        .child(text(label).text_sm())
}
