//! The composer: the card the reader writes in, and the model and effort the
//! next reply asks.

use std::path::{Path, PathBuf};
use std::time::Duration;

use aedt_provider::Effort;
use aedt_settings as settings;
use aedt_ui::{attachment, button, icon, input, menu, sizes, surface};
use nsh_base::{SubmenuEvent, theme};
use nsh_core::{file_dialog::OpenDialog, prelude::*};

pub(super) fn composer(session_id: u32, draft: Signal<String>) -> impl IntoElement {
    // The files the next message carries, in the order they were picked.
    let files = signal(Vec::<PathBuf>::new());
    let can_send = move || {
        !draft.with(|draft| draft.trim().is_empty()) || files.with(|files| !files.is_empty())
    };
    let submit = move || {
        let text = draft.get_untracked();
        let attached = files.get_untracked();
        if text.trim().is_empty() && attached.is_empty() {
            return;
        }
        aedt_bridge::ask(session_id, &text, &attached);
        draft.set(String::new());
        files.set(Vec::new());
    };
    // Why a voice attempt failed; the status bar under the card shows it until
    // it has been read.
    let notice = signal(Option::<SharedString>::None);
    // What the microphone wrote down goes into the draft, on a line of its own
    // when the draft already has words.
    effect(move || {
        let voice = aedt_bridge::voice::state().get();

        if let Some(heard) = voice.heard.clone() {
            draft.update(|draft| speak(draft, heard.as_str()));
            aedt_bridge::voice::take();
        }
        if voice.error != notice.get_untracked() {
            let error = voice.error.clone();
            notice.set(error.clone());
            if error.is_some() {
                set_timeout(Duration::from_secs(6), aedt_bridge::voice::take);
            }
        }
    });
    // Opening a conversation puts the reader in the composer: the field takes
    // the keys once it is on screen.
    let field = element_ref();
    set_timeout(Duration::from_millis(50), move || field.focus());

    view()
        .pb_2()
        .gap_2()
        .child(composer_card(draft, files, can_send, submit, field))
        // One line under the card: a failed voice attempt, and nothing else.
        .child(
            view()
                .h_5()
                .pl(px(sizes::CARD_PADDING))
                .justify_center()
                .child(when(
                    move || notice.get().is_some(),
                    move || {
                        text(move || notice.get_untracked().unwrap_or_default())
                            .text_xs()
                            .text_color(move || theme().destructive)
                    },
                )),
        )
}

/// `words` added to `draft`, after a space while the draft already has some.
fn speak(draft: &mut String, words: &str) {
    if !draft.is_empty() && !draft.ends_with(char::is_whitespace) {
        draft.push(' ');
    }
    draft.push_str(words);
}

/// The floating composer: a rounded card whose input sits on top and whose
/// round send button sits in the toolbar row below. The card is the frame:
/// the input itself is borderless, and the card's own edge is a whisper with a
/// shadow under it, not a ring that lights up.
fn composer_card(
    draft: Signal<String>,
    files: Signal<Vec<PathBuf>>,
    can_send: impl Fn() -> bool + Copy + 'static,
    submit: impl Fn() + Copy + 'static,
    field: ElementRef,
) -> impl IntoElement {
    surface::card(surface::Surface::Composer)
        .p(px(sizes::CARD_PADDING))
        .child(
            view()
                .relative()
                .gap_2()
                // What the next message carries, above the words it sends.
                .child(when(
                    move || files.with(|files| !files.is_empty()),
                    move || {
                        view().flex_row().flex_wrap().gap_1().child(each(
                            move || files.get(),
                            |path| path.clone(),
                            move |path, _| attachment_chip(path, files),
                        ))
                    },
                ))
                .child(
                    input::composer(draft, submit)
                        .label("Message")
                        .element_ref(&field),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        // The files a message carries start at the row's left
                        // end: the send stays alone at its right.
                        .child(attach_button(files, field))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(model_picker())
                                // The microphone: only a provider that writes
                                // speech down can give the button anything to do.
                                .child(when(move || settings::speech().is_some(), voice_button))
                                .child(button::icon("↑", "Send", can_send).on_press(submit)),
                        ),
                )
                // While the microphone works, the card is veiled from its own
                // middle: the meter as it listens, three dots while the provider
                // writes the recording down. The veil is thin enough to read
                // through, and takes no presses.
                .child(when(
                    move || {
                        let voice = aedt_bridge::voice::state().get();

                        voice.recording || voice.transcribing
                    },
                    listening,
                )),
        )
}

/// The button that opens the file dialog: what it picks joins the next message,
/// and the field takes the keys back once the dialog closes.
fn attach_button(files: Signal<Vec<PathBuf>>, field: ElementRef) -> impl IntoElement {
    button::quiet_icon(icon::plus(16.0), "Attach files").on_press(move || {
        let picked = pick_paths(OpenDialog::files().multiple(true).button_label("Attach"));
        spawn(async move {
            match picked.await {
                Ok(Some(paths)) => files.update(|files| {
                    for path in paths {
                        if !files.contains(&path) {
                            files.push(path);
                        }
                    }
                }),
                Ok(None) => {}
                Err(why) => eprintln!("attachment: the file dialog failed: {why}"),
            }
            field.focus();
        });
    })
}

/// One file the next message carries: its name, and a press takes it out.
fn attachment_chip(path: ReadSignal<PathBuf>, files: Signal<Vec<PathBuf>>) -> impl IntoElement {
    let name = path.with_untracked(|path| file_name(path));
    let picked = path.with_untracked(|path| path.clone());

    attachment::chip(name)
        .on_press(move || files.update(|files| files.retain(|file| *file != picked)))
}

/// What a chip calls `path`: its file name, or the path whole when it has none.
fn file_name(path: &Path) -> SharedString {
    path.file_name()
        .map_or_else(
            || path.to_string_lossy().into_owned(),
            |name| name.to_string_lossy().into_owned(),
        )
        .into()
}

/// How many bars the meter has.
const WAVE_BARS: usize = 13;

/// The bars the field shows while the microphone listens: the middle one
/// tallest, and the ends a fifth of it.
const WAVE_PROFILE: [f32; WAVE_BARS] = [
    0.15, 0.3, 0.5, 0.78, 1.0, 0.62, 0.92, 0.62, 1.0, 0.78, 0.5, 0.3, 0.15,
];

/// One bar's width, the gap beside it, and the shortest and tallest it gets.
const WAVE_BAR: f32 = 3.0;
const WAVE_GAP: f32 = 2.0;
const WAVE_MIN: f32 = 5.0;
const WAVE_MAX: f32 = 36.0;

/// The dots that wait for the provider's answer: how many, how round, how far
/// apart, and how fast the wave runs along them (a fifth of a radian a frame,
/// so a turn takes about a second).
const DOT_COUNT: u8 = 3;
const DOT_SIZE: f32 = 6.0;
const DOT_GAP: f32 = 5.0;
const DOT_STEP: f32 = 0.2;
const DOT_TURN_3: f32 = std::f32::consts::TAU / 3.0;

/// The veil over the card while the microphone works, its meter or its dots in
/// the card's middle. The veil is thin enough to read the card through, and
/// takes no presses: what is under it stays in sight and still works.
fn listening() -> impl IntoElement {
    let voice = aedt_bridge::voice::state();

    view()
        .absolute()
        .top(px(0.0))
        .bottom(px(0.0))
        .left(px(0.0))
        .right(px(0.0))
        .items_center()
        .justify_center()
        .bg(move || theme().card.opacity(0.8))
        .pointer_events(PointerEvents::None)
        .child(when(move || voice.get().recording, meter))
        .child(when(move || voice.get().transcribing, dots))
}

/// The bars of the meter: the one thing the veil shows while the microphone
/// listens.
fn meter() -> impl IntoElement {
    let meter = aedt_bridge::voice::meter();
    let mut bars = view().flex_row().items_center().gap(px(WAVE_GAP));
    for index in 0..WAVE_BARS {
        bars = bars.child(bar(index, meter));
    }

    bars
}

/// Three dots, the next one lit in turn, while the provider writes the
/// recording down.
fn dots() -> impl IntoElement {
    let dots = aedt_bridge::voice::dots();
    let mut row = view().flex_row().items_center().gap(px(DOT_GAP));
    for index in 0..DOT_COUNT {
        row = row.child(dot(index, dots));
    }

    row
}

/// One dot of the three: its light comes and goes as the wave passes it, so
/// they brighten and dim rather than switch.
fn dot(index: u8, dots: Signal<aedt_bridge::voice::Dots>) -> impl IntoElement {
    view()
        .w(px(DOT_SIZE))
        .h(px(DOT_SIZE))
        .rounded_full()
        .bg(move || theme().muted_foreground)
        .opacity(move || lit(index, dots.get().0))
}

/// How lit the dot at `index` is at `frame`: a wave that runs along the three,
/// each a third of a turn behind the one before it.
fn lit(index: u8, frame: u32) -> f32 {
    let phase = frame as f32 * DOT_STEP + index as f32 * DOT_TURN_3;
    let wave = (phase.sin() * 0.5 + 0.5).powi(2);

    0.25 + 0.75 * wave
}

/// One bar, `index` from the start, as tall as the meter and its place in the
/// profile say.
fn bar(index: usize, meter: Signal<aedt_bridge::voice::Meter>) -> impl IntoElement {
    view()
        .w(px(WAVE_BAR))
        .rounded_full()
        .bg(move || theme().muted_foreground)
        .h(move || px(height(index, meter.get())))
}

/// How tall the bar at `index` stands for `meter`: the profile, the loudness,
/// and a ripple that travels along the row as the measurements come.
fn height(index: usize, meter: aedt_bridge::voice::Meter) -> f32 {
    let level = 0.12 + 0.88 * meter.level.clamp(0.0, 1.0);
    let ripple = 0.55 + 0.45 * (meter.frame as f32 * 0.55 + index as f32 * 1.1).sin();

    (WAVE_MIN + (WAVE_MAX - WAVE_MIN) * WAVE_PROFILE[index] * level * ripple)
        .clamp(WAVE_MIN, WAVE_MAX)
}

/// The microphone beside the model: a press starts recording, and the next one
/// has the provider write down what was said.
fn voice_button() -> impl IntoElement {
    let voice = aedt_bridge::voice::state();

    button::ghost_icon(icon::mic(16.0), "Voice input")
        .text_color(move || {
            if voice.get().recording {
                theme().primary
            } else {
                theme().muted_foreground
            }
        })
        .accessibility_label(move || {
            let label = if voice.get().recording {
                "Stop recording"
            } else {
                "Voice input"
            };

            Some(SharedString::from(label))
        })
        .disabled(move || voice.get().transcribing)
        .on_press(aedt_bridge::voice::toggle)
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
