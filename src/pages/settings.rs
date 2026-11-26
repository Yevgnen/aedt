//! The settings dialog: its categories on the left, the chosen one's content
//! on the right. Every provider is configured here, with the models it lists.

use aedt_ui::{badge, button, collapsible, field, icon, list_item, menu, sizes, switch, tokens};
use nsh_base::{dialog as base_dialog, theme};
use nsh_core::prelude::*;

use aedt_settings::{self as settings, ProviderKind};

/// The settings, presented while `open` holds and dismissed when it is let go,
/// by Escape or a press on the dim behind them.
pub(crate) fn dialog(open: Signal<bool>) -> impl IntoElement {
    let dismiss = move || open.set(false);

    when(
        move || open.get(),
        move || {
            base_dialog()
                .animation(ModalAnimation::Fade)
                .label("Settings")
                .on_cancel(dismiss)
                .flex_1()
                .child(
                    view()
                        .absolute()
                        .inset_0()
                        .bg(tokens::scrim())
                        .on_press(dismiss),
                )
                .child(card())
        },
    )
}

/// The dialog's card: the categories, then the chosen one.
fn card() -> impl IntoElement {
    view()
        .m(auto())
        .flex_row()
        .w(px(sizes::DIALOG_W))
        .h(px(sizes::DIALOG_H))
        .rounded_2xl()
        .border_1()
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .text_color(move || theme().foreground)
        .shadow(tokens::floating_shadow())
        .child(categories())
        .child(content())
}

/// The left column: the dialog's title, then its categories. The only one so
/// far is the providers', which is the chosen one.
fn categories() -> impl IntoElement {
    view()
        .w(px(sizes::CATEGORIES_W))
        .flex_none()
        .p_2()
        .gap_1()
        .border_r_1()
        .border_r_color(move || theme().border)
        .child(
            text("Settings")
                .px(px(sizes::LIST_ROW_INSET))
                .pt_2()
                .pb_3()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(list_item::nav_with_mark(
            icon::database(14.0),
            "Models",
            || true,
        ))
}

/// The chosen category's content: its heading, then the page it shows.
fn content() -> impl IntoElement {
    view().flex_1().min_w(px(0.0)).min_h(px(0.0)).child(
        scroll()
            .flex_1()
            .min_h(px(0.0))
            .child(view().w_full().p_6().gap_5().child(heading()).child(page())),
    )
}

/// The pane's heading: what it is, and what it asks the reader for.
fn heading() -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        .child(
            text(move || {
                if settings::draft().is_some() {
                    SharedString::from("Add model provider")
                } else {
                    SharedString::from("Models")
                }
            })
            .text_lg()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(move || theme().foreground),
        )
        .child(when(
            move || settings::draft().is_none(),
            || {
                text("Enter your API keys to use models from the following providers.")
                    .text_sm()
                    .text_color(move || theme().muted_foreground)
            },
        ))
}

/// The page: the action that adds a provider, the ones the reader kept, or the
/// provider being added.
fn page() -> impl IntoElement {
    view()
        .w_full()
        .gap_5()
        .child(when(move || settings::draft().is_none(), add_provider))
        .child(when(
            move || settings::draft().is_none(),
            || {
                view().w_full().gap_2().child(each(
                    settings::saved_providers,
                    |id| *id,
                    |id, _| provider_card(id.get()),
                ))
            },
        ))
        .child(when(
            move || settings::draft().is_some(),
            || add_form(settings::draft().unwrap_or_default()),
        ))
}

/// The action that starts adding a provider: a press opens its form, where its
/// name and kind are chosen.
fn add_provider() -> impl IntoElement {
    button::add("Add model provider").on_press(|| {
        settings::add_provider(ProviderKind::OpenAiChatCompletions);
    })
}

/// The provider being added: its name and kind, its fields, the models they can
/// list, and the commands that keep it or drop it.
fn add_form(id: u32) -> impl IntoElement {
    view()
        .w_full()
        .gap_5()
        .child(
            view()
                .w_full()
                .gap_3()
                .child(name_field(id))
                .child(api_field(id))
                .child(provider_fields(id)),
        )
        .child(models(id))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_end()
                .gap_2()
                .child(button::ghost("Cancel").on_press(move || settings::cancel_provider(id)))
                .child(
                    button::secondary("Save")
                        .disabled(move || !settings::can_save(id))
                        .on_press(move || settings::save_provider(id)),
                ),
        )
}

/// What the provider being added is called, which it keeps once saved.
fn name_field(id: u32) -> impl IntoElement {
    field::field(
        "Name",
        move || settings::name(id).as_str().to_owned(),
        move |name| settings::set_name(id, name),
        || {},
        field::Mask::Plain,
    )
}

/// What API a provider speaks: a field that opens the APIs the app speaks.
fn api_field(id: u32) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        .child(field::label("API"))
        .child(menu::menu(
            menu::select_trigger(move || kind_label(settings::kind(id))),
            Placement::Bottom,
            Align::Start,
            view()
                .gap_1()
                .child(
                    menu::item(icon::database(14.0), "Responses").on_select(move || {
                        settings::select_kind(id, ProviderKind::OpenAiResponses);
                    }),
                )
                .child(
                    menu::item(icon::database(14.0), "Chat completions").on_select(move || {
                        settings::select_kind(id, ProviderKind::OpenAiChatCompletions);
                    }),
                ),
        ))
}

/// One provider, folded to its name until it is asked for: what it is, how to
/// reach it, and the models it lists.
fn provider_card(id: u32) -> impl IntoElement {
    collapsible::section(
        move || settings::name(id),
        // The kind, unless the name is the kind's own.
        when(
            move || settings::name(id) != kind_label(settings::kind(id)),
            move || badge::badge(move || kind_label(settings::kind(id))),
        ),
        provider_body(id),
    )
    .w_full()
}

/// How a provider is reached: its URL and the key its requests carry, with the
/// environment variable the key came from when it came from one.
fn provider_fields(id: u32) -> impl IntoElement {
    view()
        .w_full()
        .gap_3()
        .child(field::field(
            "API URL",
            move || settings::base_url(id),
            move |url| settings::set_base_url(id, url),
            move || settings::refresh_models(id),
            field::Mask::Plain,
        ))
        .child(
            view()
                .w_full()
                .gap_1()
                .child(field::field_with_trailing(
                    "API Key",
                    move || settings::api_key(id),
                    move |key| settings::set_api_key(id, key),
                    move || settings::refresh_models(id),
                    field::Mask::Secret,
                    button::ghost_icon(icon::rotate_ccw(14.0), "Reset the key")
                        .on_press(move || settings::reset_api_key(id)),
                ))
                .child(when(
                    move || settings::env_key(id).is_some(),
                    move || {
                        text(move || {
                            settings::env_key(id)
                                .map(|variable| {
                                    format!("Using the {variable} environment variable")
                                })
                                .unwrap_or_default()
                        })
                        .text_xs()
                        .text_color(tokens::meta)
                    },
                )),
        )
}

/// What a provider's section holds once it opens.
fn provider_body(id: u32) -> impl IntoElement {
    view()
        .w_full()
        .gap_4()
        .child(api_field(id))
        .child(provider_fields(id))
        .child(models(id))
}

/// What API a provider of `kind` speaks, as its picker says it.
fn kind_label(kind: Option<ProviderKind>) -> SharedString {
    match kind {
        Some(ProviderKind::OpenAiResponses) => "Responses".into(),
        Some(ProviderKind::OpenAiChatCompletions) => "Chat completions".into(),
        None => SharedString::default(),
    }
}

/// One provider's models: their heading with the ask, how the last ask went,
/// then a row per model with its switch.
fn models(id: u32) -> impl IntoElement {
    view()
        .w_full()
        .gap_2()
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    text("Models")
                        .flex_1()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    button::ghost_icon(icon::checks(14.0), "Enable or disable every model")
                        .text_color(move || {
                            if settings::all_models_enabled(id) {
                                theme().foreground
                            } else {
                                theme().muted_foreground
                            }
                        })
                        .disabled(move || settings::models(id).is_empty())
                        .on_press(move || {
                            let on = !settings::all_models_enabled(id);
                            settings::set_all_models_enabled(id, on);
                        }),
                )
                .child(
                    button::ghost_icon(icon::refresh(14.0), "Refresh models")
                        .disabled(move || !settings::can_refresh(id))
                        .on_press(move || settings::refresh_models(id)),
                ),
        )
        .child(when(
            move || settings::loading(id),
            || note("Loading models…"),
        ))
        .child(when(
            move || settings::error(id).is_some(),
            move || {
                text(move || settings::error(id).unwrap_or_default())
                    .w_full()
                    .text_sm()
                    .number_of_lines(3)
                    .text_color(move || theme().destructive)
            },
        ))
        .child(when(
            move || settings::listed_none(id),
            || note("No models"),
        ))
        .child(view().w_full().gap_1().child(each(
            move || settings::models(id),
            |model| model.id.clone(),
            move |model, _| model_row(id, model),
        )))
}

/// A quiet line about the models: loading, or none to show.
fn note(line: &'static str) -> impl IntoElement {
    text(line).text_sm().text_color(tokens::meta)
}

/// A row of that list: the model's name, and the switch that offers it.
fn model_row(provider: u32, model: ReadSignal<settings::Model>) -> impl IntoElement {
    let label = move || model.with(|model| model.label.clone());
    let enabled = move || model.with(|model| model.enabled);

    view()
        .w_full()
        .flex_row()
        .items_center()
        .gap_3()
        .min_h(px(sizes::LIST_ROW))
        .px_2()
        .rounded(move || theme().radius)
        .child(
            text(label)
                .flex_1()
                .min_w(px(0.0))
                .number_of_lines(1)
                .text_sm()
                .text_color(move || theme().foreground),
        )
        .child(switch::toggle(label, enabled, move |on| {
            let id = model.with(|model| model.id.clone());
            settings::set_model_enabled(provider, id.as_str(), on);
        }))
}
