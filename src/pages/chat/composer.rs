//! The composer: the card the reader writes in, and the model and effort the
//! next reply asks.

use std::time::Duration;

use aedt_provider::Effort;
use aedt_settings as settings;
use aedt_ui::{badge, button, input, menu, sizes, surface};
use nsh_base::SubmenuEvent;
use nsh_core::prelude::*;

pub(super) fn composer(session_id: u32, draft: Signal<String>) -> impl IntoElement {
    let can_send = move || draft.with(|draft| !draft.trim().is_empty());
    let submit = move || {
        let text = draft.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        aedt_bridge::ask(session_id, &text);
        draft.set(String::new());
    };
    // Opening a conversation puts the reader in the composer: the field takes
    // the keys once it is on screen.
    let field = element_ref();
    set_timeout(Duration::from_millis(50), move || field.focus());

    view()
        .pb_2()
        .gap_2()
        .child(composer_card(draft, can_send, submit, field))
        // One line reserved for the status bar; it holds no text yet.
        .child(view().h_5().pl(px(sizes::CARD_PADDING)).justify_center())
}

/// The floating composer: a rounded card whose input sits on top and whose
/// round send button sits in the toolbar row below. The card is the frame:
/// the input itself is borderless, and the card's own edge is a whisper with a
/// shadow under it, not a ring that lights up.
fn composer_card(
    draft: Signal<String>,
    can_send: impl Fn() -> bool + Copy + 'static,
    submit: impl Fn() + Copy + 'static,
    field: ElementRef,
) -> impl IntoElement {
    surface::card(surface::Surface::Composer)
        .p(px(sizes::CARD_PADDING))
        .gap_2()
        .child(
            input::composer(draft, submit)
                .label("Message")
                .element_ref(&field),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_end()
                .gap_2()
                .child(model_picker())
                .child(button::icon("↑", "Send", can_send).on_press(submit)),
        )
}

/// The model and the effort the next reply asks: each row names what it holds,
/// and the list it opens holds the choices.
fn model_picker() -> impl IntoElement {
    let query = signal(String::new());
    // The field that takes the keys as the list opens.
    let search = element_ref();
    // The menu's rows are plain presses, which leave the search's keys alone:
    // the app closes it when one is chosen.
    let open = signal(false);

    menu::controlled(
        open,
        menu::label_trigger(
            badge::tag(move || settings::selected().provider_name),
            move || settings::selected().label,
            move || SharedString::from(effort_label(settings::effort())),
        )
        .disabled(move || settings::enabled_models().is_empty()),
        Placement::Top,
        Align::End,
        view()
            .child(
                menu::submenu(
                    "Model",
                    move || settings::selected().label,
                    view()
                        .min_w(px(sizes::PICKER_MIN_W))
                        .gap_1()
                        // The row that opened it takes no focus (a pointer
                        // over it would take the field's), so the keys stay
                        // in the field as the list is searched.
                        .child(
                            menu::search(query, "Search models…")
                                .auto_focus(true)
                                .element_ref(&search),
                        )
                        // The models scroll in their own view under the field,
                        // so a wheel over them never takes the field's keys.
                        .child(
                            scroll()
                                .max_h(px(sizes::PICKER_MAX_H))
                                .scrollbar(false)
                                .child(model_choices(query, open)),
                        ),
                )
                .on_open_change(move |event: &SubmenuEvent| {
                    let SubmenuEvent::OpenChange(open, _) = event;
                    if *open {
                        // The popup is made in this commit: the field takes
                        // the keys once it is there.
                        set_timeout(Duration::from_millis(50), move || search.focus());
                    } else {
                        // Each opening searches from the start again.
                        query.set(String::new());
                    }
                }),
            )
            .child(menu::submenu(
                "Effort",
                move || SharedString::from(effort_label(settings::effort())),
                effort_choices(),
            )),
    )
}

/// The models the settings offer, grouped under the provider that serves them
/// and narrowed to `query` (every one while it is empty), the chosen one
/// checked.
fn model_choices(query: Signal<String>, open: Signal<bool>) -> impl IntoElement {
    each(
        move || {
            let query = query.with(|query| query.trim().to_lowercase());
            let mut sections: Vec<ModelSection> = Vec::new();
            for choice in settings::enabled_models() {
                if !query.is_empty() && !choice.label.as_str().to_lowercase().contains(&query) {
                    continue;
                }
                match sections.last_mut() {
                    Some(section) if section.provider == choice.provider => {
                        section.choices.push(choice);
                    }
                    _ => sections.push(ModelSection {
                        provider: choice.provider,
                        name: choice.provider_name.clone(),
                        choices: vec![choice],
                    }),
                }
            }
            sections
        },
        |section| section.provider,
        move |section, _| {
            view()
                .w_full()
                .gap_1()
                .child(menu::heading(move || {
                    section.with(|section| section.name.clone())
                }))
                .child(each(
                    move || section.with(|section| section.choices.clone()),
                    |choice| choice.id.clone(),
                    move |choice, _| {
                        menu::option(
                            move || choice.with(|choice| choice.label.clone()),
                            move || {
                                let selected = settings::selected();
                                choice.with(|choice| {
                                    choice.provider == selected.provider && choice.id == selected.id
                                })
                            },
                        )
                        .on_press(move || {
                            let (provider, id) =
                                choice.with(|choice| (choice.provider, choice.id.clone()));
                            settings::select_model(provider, id.as_str());
                            open.set(false);
                        })
                    },
                ))
        },
    )
}

/// The models one provider serves, as the picker lists them under its name.
#[derive(Clone, PartialEq)]
struct ModelSection {
    provider: u32,
    name: SharedString,
    choices: Vec<settings::Choice>,
}

/// The efforts a reply can ask for, the weakest first.
fn effort_choices() -> impl IntoElement {
    view()
        .min_w(px(sizes::EFFORT_W))
        .gap_1()
        .child(effort_choice(Effort::Default))
        .child(effort_choice(Effort::Low))
        .child(effort_choice(Effort::Medium))
        .child(effort_choice(Effort::High))
}

fn effort_choice(effort: Effort) -> impl IntoElement {
    menu::choice(effort_label(effort), move || settings::effort() == effort)
        .on_select(move || settings::select_effort(effort))
}

/// What `effort` is called in the picker.
fn effort_label(effort: Effort) -> &'static str {
    match effort {
        Effort::Default => "Default",
        Effort::Low => "Low",
        Effort::Medium => "Medium",
        Effort::High => "High",
    }
}
