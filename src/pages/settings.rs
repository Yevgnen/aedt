//! The settings dialog: its categories on the left, the chosen one's content
//! on the right. Every provider is configured here, with the models it lists.

use aedt_ui::{
    badge, button, collapsible, field, icon, list_item, menu, sizes, surface, switch, tokens,
};
use nsh_base::{dialog as base_dialog, theme};
use nsh_core::prelude::*;

use aedt_settings::{self as settings, ProviderKind, Thinking};

/// Which page of the settings shows.
#[derive(Clone, Copy, PartialEq)]
enum Category {
    Models,
    Mcp,
    Conversation,
}

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
    let category = signal(Category::Models);

    surface::card(surface::Surface::Dialog)
        .m(auto())
        .flex_row()
        .w(px(sizes::DIALOG_W))
        .h(px(sizes::DIALOG_H))
        .text_color(move || theme().foreground)
        .child(categories(category))
        .child(content(category))
}

/// The left column: the dialog's title, then its categories.
fn categories(category: Signal<Category>) -> impl IntoElement {
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
        .child(
            list_item::nav_with_mark(icon::database(14.0), "Models", move || {
                category.get() == Category::Models
            })
            .on_press(move || category.set(Category::Models)),
        )
        .child(
            list_item::nav_with_mark(icon::wrench(14.0), "MCP", move || {
                category.get() == Category::Mcp
            })
            .on_press(move || category.set(Category::Mcp)),
        )
        .child(
            list_item::nav_with_mark(icon::bulb(14.0), "Conversation", move || {
                category.get() == Category::Conversation
            })
            .on_press(move || category.set(Category::Conversation)),
        )
}

/// The chosen category's content: its heading, then the page it shows.
fn content(category: Signal<Category>) -> impl IntoElement {
    view().flex_1().min_w(px(0.0)).min_h(px(0.0)).child(
        scroll().flex_1().min_h(px(0.0)).child(
            view()
                .w_full()
                .p_6()
                .gap_5()
                .child(heading(category))
                .child(when(
                    move || category.get() == Category::Models,
                    models_page,
                ))
                .child(when(move || category.get() == Category::Mcp, mcp_page))
                .child(when(
                    move || category.get() == Category::Conversation,
                    conversation_page,
                )),
        ),
    )
}

/// The pane's heading: what it is, and what it asks the reader for.
fn heading(category: Signal<Category>) -> impl IntoElement {
    view()
        .w_full()
        .gap_1()
        .child(
            text(move || match category.get() {
                Category::Models if settings::draft().is_some() => {
                    SharedString::from("Add model provider")
                }
                Category::Models => SharedString::from("Models"),
                Category::Mcp if settings::mcp_draft().is_some() => {
                    SharedString::from("Add MCP server")
                }
                Category::Mcp => SharedString::from("MCP"),
                Category::Conversation => SharedString::from("Conversation"),
            })
            .text_lg()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(move || theme().foreground),
        )
        .child(when(
            move || category.get() == Category::Models && settings::draft().is_none(),
            || {
                text("Enter your API keys to use models from the following providers.")
                    .text_sm()
                    .text_color(move || theme().muted_foreground)
            },
        ))
        .child(when(
            move || category.get() == Category::Mcp && settings::mcp_draft().is_none(),
            || {
                text("Local servers whose tools the agent can call.")
                    .text_sm()
                    .text_color(move || theme().muted_foreground)
            },
        ))
}

/// The conversation page: how a reply's thinking shows while it arrives, and
/// how wide the conversation's column runs.
fn conversation_page() -> View {
    view()
        .w_full()
        .gap_5()
        .child(
            view()
                .w_full()
                .gap_1()
                .child(field::label("Thinking"))
                .child(menu::menu(
                    menu::select_trigger(move || thinking_label(settings::thinking())),
                    Placement::Bottom,
                    Align::Start,
                    view()
                        .gap_1()
                        .child(thinking_choice(Thinking::Collapsed))
                        .child(thinking_choice(Thinking::Expanded))
                        .child(thinking_choice(Thinking::Auto))
                        .child(thinking_choice(Thinking::Preview)),
                )),
        )
        .child(
            view()
                .w_full()
                .gap_1()
                .child(field::label("Side margin"))
                .child(menu::menu(
                    menu::select_trigger(move || margin_label(settings::margin())),
                    Placement::Bottom,
                    Align::Start,
                    view()
                        .gap_1()
                        .child(margin_choice(10))
                        .child(margin_choice(15))
                        .child(margin_choice(20))
                        .child(margin_choice(25)),
                )),
        )
}

/// One blank the conversation may keep on each side, checked while it is the
/// chosen one.
fn margin_choice(percent: u32) -> impl IntoElement {
    menu::choice(margin_label(percent), move || settings::margin() == percent)
        .on_select(move || settings::select_margin(percent))
}

/// A blank, as the settings say it: a share of the conversation's width.
fn margin_label(percent: u32) -> SharedString {
    format!("{percent}%").into()
}

/// One way a reply's thinking may show, checked while it is the chosen one.
fn thinking_choice(thinking: Thinking) -> impl IntoElement {
    menu::choice(thinking_label(thinking), move || {
        settings::thinking() == thinking
    })
    .on_select(move || settings::select_thinking(thinking))
}

/// What `thinking` is called in the settings.
fn thinking_label(thinking: Thinking) -> SharedString {
    match thinking {
        Thinking::Collapsed => "Collapsed",
        Thinking::Expanded => "Expanded",
        Thinking::Auto => "Automatic",
        Thinking::Preview => "Preview",
    }
    .into()
}

/// The models page: the action that adds a provider, the ones the reader kept,
/// or the provider being added.
fn models_page() -> View {
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
                        // The button's own state is what screen readers read:
                        // selected is on, unselected is off.
                        .selected(move || settings::all_models_enabled(id))
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

/// The MCP page: the action that adds a server, the ones the reader kept, or the
/// server being added.
fn mcp_page() -> View {
    view()
        .w_full()
        .gap_5()
        .child(when(move || settings::mcp_draft().is_none(), add_mcp))
        .child(when(
            move || settings::mcp_draft().is_none(),
            || {
                view().w_full().gap_2().child(each(
                    settings::mcps,
                    |id| *id,
                    |id, _| mcp_card(id.get()),
                ))
            },
        ))
        .child(when(
            move || settings::mcp_draft().is_some(),
            || mcp_form(settings::mcp_draft().unwrap_or_default()),
        ))
}

/// The action that starts adding a server: a press opens its form, where what
/// starts it is filled in.
fn add_mcp() -> impl IntoElement {
    button::add("Add MCP server").on_press(settings::add_mcp)
}

/// The server being added: what starts it, then the commands that keep it or
/// drop it.
fn mcp_form(id: u32) -> impl IntoElement {
    view().w_full().gap_5().child(mcp_fields(id)).child(
        view()
            .w_full()
            .flex_row()
            .items_center()
            .justify_end()
            .gap_2()
            .child(button::ghost("Cancel").on_press(move || settings::cancel_mcp(id)))
            .child(
                button::secondary("Save")
                    .disabled(move || !settings::can_save_mcp(id))
                    .on_press(move || settings::save_mcp(id)),
            ),
    )
}

/// One server, folded to its name until it is asked for: the switch that
/// starts it, before the chevron, and what starts it once the section opens.
fn mcp_card(id: u32) -> impl IntoElement {
    collapsible::section_with_trailing(
        move || mcp_title(id),
        switch::toggle(
            move || mcp_title(id),
            move || settings::mcp_enabled(id),
            move |on| settings::set_mcp_enabled(id, on),
        ),
        mcp_fields(id),
    )
    .w_full()
}

/// How a server starts: what it is called, the program, what the program is
/// given as arguments, and the environment it runs with.
fn mcp_fields(id: u32) -> impl IntoElement {
    view()
        .w_full()
        .gap_3()
        .child(field::field(
            "Name",
            move || settings::mcp_name(id).as_str().to_owned(),
            move |name| settings::set_mcp_name(id, name),
            || {},
            field::Mask::Plain,
        ))
        .child(field::field(
            "Command",
            move || settings::mcp_command(id),
            move |command| settings::set_mcp_command(id, command),
            || {},
            field::Mask::Plain,
        ))
        .child(field::multiline(
            "Arguments",
            3,
            move || settings::mcp_args(id),
            move |args| settings::set_mcp_args(id, args),
        ))
        .child(field::multiline(
            "Environment",
            4,
            move || settings::mcp_env(id),
            move |env| settings::set_mcp_env(id, env),
        ))
}

/// What a server's header says: its name, else the program it runs, else that
/// it has neither yet.
fn mcp_title(id: u32) -> SharedString {
    let name = settings::mcp_name(id);
    if !name.as_str().trim().is_empty() {
        return name;
    }
    let command = settings::mcp_command(id);
    if !command.trim().is_empty() {
        return command.trim().to_owned().into();
    }

    SharedString::from("New server")
}
