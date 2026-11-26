//! The app's menus: the row that opens one, the card its items open in, an
//! item of it, and a picker's row and trigger. `nsh/base` owns the behavior
//! (placement, focus, keyboard navigation, dismissal); the look is here.

use nsh_base::{
    ButtonElement, Input, MenuElement, MenuEvent, MenuItemElement, SubmenuElement, button, input,
    menu as base_menu, menu_item as base_item, submenu as base_submenu, theme,
};
use nsh_core::prelude::*;

use crate::{icon, motion, sizes, tokens};

/// A menu that `trigger` opens, its `items` in the app's floating card, on the
/// side of the trigger `placement` names and lined up with the edge `align`
/// names (the other side, when only that one has room).
pub fn menu(
    trigger: impl IntoElement,
    placement: Placement,
    align: Align,
    items: impl IntoElement,
) -> MenuElement {
    base_menu()
        .placement(placement)
        .align(align)
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

/// The same menu with `open` in the app's hands: a row of it can close it,
/// as an item of an uncontrolled one closes it by itself.
pub fn controlled(
    open: Signal<bool>,
    trigger: impl IntoElement,
    placement: Placement,
    align: Align,
    items: impl IntoElement,
) -> MenuElement {
    menu(trigger, placement, align, items)
        .open(move || Some(open.get()))
        .on_open_change(move |event: &MenuEvent| {
            if let MenuEvent::OpenChange(next, _) = event {
                open.set(*next);
            }
        })
}

/// A row that opens a menu: a mark and its label, in the app's quiet chrome.
/// The menu opens it on a press and with the keyboard; it takes no press of
/// its own.
pub fn trigger(mark: impl IntoElement, label: impl IntoReactive<SharedString>) -> ButtonElement {
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

/// A row at a control's edge that names its value and opens its picker: a
/// small quiet label, the note, and the chevron that opens the picker.
/// The menu opens it as it does [`trigger`].
pub fn label_trigger(
    value: impl IntoReactive<SharedString>,
    note: impl IntoReactive<SharedString>,
) -> ButtonElement {
    button()
        .flex_row()
        .items_center()
        .gap_1()
        .px_1()
        .py_0p5()
        .rounded_md()
        .text_color(move || theme().muted_foreground)
        .hover(|s| s.bg(move || theme().muted))
        .expanded(|s| s.bg(move || theme().accent))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .disabled_style(|s| s.opacity(0.4))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(text(value).text_size(px(13.0)))
        .child(text(note).text_size(px(13.0)).text_color(tokens::caption))
        .child(icon::chevron_down(12.0))
}

/// A picker's section heading: the name of the provider whose models follow,
/// quiet and small over them.
pub fn heading(label: impl IntoReactive<SharedString>) -> impl IntoElement {
    text(label)
        .w_full()
        .px(px(sizes::LIST_ROW_INSET))
        .pt_1()
        .text_xs()
        .text_color(move || theme().muted_foreground)
}

/// An item of one of those menus: a mark and its label, in the app's row look.
/// Choosing it closes the menu.
pub fn item(mark: impl IntoElement, label: impl IntoReactive<SharedString>) -> MenuItemElement {
    row(base_item()).child(mark).child(text(label).text_sm())
}

/// A picker's row: its label, and a check at its end while `chosen` holds. The
/// caller adds `.on_select`, which closes the menu as an item's does.
pub fn choice(
    label: impl IntoReactive<SharedString>,
    chosen: impl Fn() -> bool + Copy + 'static,
) -> MenuItemElement {
    row(base_item())
        .role(AccessibilityRole::MenuItemRadio)
        .checked(move || Some(chosen()))
        .child(
            text(label)
                .flex_1()
                .min_w(px(0.0))
                .number_of_lines(1)
                .text_sm(),
        )
        .child(when(chosen, || icon::check(14.0)))
}

/// A picker's row that leaves the keyboard focus alone: a list under a search
/// field keeps the keys while the pointer moves over its rows, which a menu
/// item would take. The caller adds `.on_press`, and the check `chosen` draws.
pub fn option(
    label: impl IntoReactive<SharedString>,
    chosen: impl Fn() -> bool + Copy + 'static,
) -> ButtonElement {
    crate::button::bare()
        .role(AccessibilityRole::MenuItemRadio)
        .w_full()
        .flex_row()
        .items_center()
        .gap_2()
        .min_h(px(sizes::LIST_ROW))
        .px(px(sizes::LIST_ROW_INSET))
        .rounded(move || theme().radius)
        .text_color(move || theme().foreground)
        .hover(|s| s.bg(move || theme().muted))
        .active(|s| s.bg(move || theme().accent))
        .transition_colors()
        .duration(motion::row_duration())
        .ease_out()
        .child(
            text(label)
                .flex_1()
                .min_w(px(0.0))
                .number_of_lines(1)
                .text_sm(),
        )
        .child(when(chosen, || icon::check(14.0)))
}

/// A value that opens a menu of values, shaped like a form field: the value,
/// then a chevron, in the field's edge.
pub fn select_trigger(value: impl IntoReactive<SharedString>) -> ButtonElement {
    button()
        .w_full()
        .flex_row()
        .items_center()
        .gap_2()
        .min_h(px(sizes::CONTROL))
        .px_2()
        .rounded(move || theme().radius)
        .border_1()
        .border_color(move || theme().border)
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .hover(|s| s.bg(move || theme().muted))
        .expanded(|s| s.border_color(move || theme().ring))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .transition_colors()
        .duration(motion::control_duration())
        .ease_out()
        .child(text(value).flex_1().min_w(px(0.0)).text_sm())
        .child(
            view()
                .flex_none()
                .text_color(move || theme().muted_foreground)
                .child(icon::chevron_down(12.0)),
        )
}

/// A row of a menu that opens its `items` beside it: its label, the value it
/// holds, and a chevron. Its items open in the menu's own card.
pub fn submenu(
    label: impl IntoReactive<SharedString>,
    value: impl IntoReactive<SharedString>,
    items: impl IntoElement,
) -> SubmenuElement {
    base_submenu()
        .offset(px(4.0))
        .trigger(
            view()
                // A menu's items take the keyboard focus as the pointer moves
                // over them, and the row under the pointer when a submenu
                // opens would take it from a field of the submenu's own (a
                // picker's search): the row opens on a press and on hover, and
                // takes no focus of its own.
                .focusable(false)
                .flex_row()
                .items_center()
                .gap_2()
                .min_h(px(sizes::LIST_ROW))
                .px(px(sizes::LIST_ROW_INSET))
                .rounded(move || theme().radius)
                .text_color(move || theme().muted_foreground)
                .hover(|s| s.bg(move || theme().muted))
                .expanded(|s| s.bg(move || theme().accent))
                .transition_colors()
                .duration(motion::row_duration())
                .ease_out()
                .child(
                    text(label)
                        .flex_1()
                        .text_sm()
                        .text_color(move || theme().foreground),
                )
                .child(text(value).text_sm())
                .child(icon::chevron_right(12.0)),
        )
        .child(items)
}

/// The field over a picker's rows: one quiet line that narrows the list as it
/// is typed, with no frame of its own.
pub fn search(query: Signal<String>, placeholder: &'static str) -> Input {
    input()
        .value(move || query.get())
        .on_change(move |text: &str| query.set(text.to_owned()))
        .placeholder(placeholder)
        .auto_correct(false)
        .min_h(px(sizes::LIST_ROW))
        .px(px(sizes::LIST_ROW_INSET))
        .text_sm()
        .text_color(move || theme().foreground)
}

/// A menu row's look, whichever content it holds.
fn row(item: MenuItemElement) -> MenuItemElement {
    item.flex_row()
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
}
