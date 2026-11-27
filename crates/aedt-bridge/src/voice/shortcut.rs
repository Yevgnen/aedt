//! The reader's own keys for the microphone: the right Option starts and ends
//! a recording, and Escape ends the one in flight.
//!
//! AppKit reports a modifier key only as `flagsChanged`, and only to the
//! monitor that runs before the responder chain, so the presses come from a
//! local event monitor on the main thread. A signal is the engine's to write,
//! so the monitor sends each press through a channel and the engine takes it.

#[cfg(target_os = "macos")]
use std::{
    sync::{
        OnceLock,
        mpsc::{self, Sender},
    },
    time::Duration,
};

#[cfg(target_os = "macos")]
use nsh_core::prelude::*;

/// What a key asks of the microphone.
#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
enum Press {
    /// The right Option: start a recording, or end the one in flight.
    Toggle,
    /// Escape: end the recording, if one is running.
    Stop,
}

/// Calls `toggle` on the engine at each press of the shortcut, once for the
/// app.
#[cfg(target_os = "macos")]
pub(crate) fn watch() {
    static WATCHING: OnceLock<()> = OnceLock::new();
    if WATCHING.set(()).is_err() {
        return;
    }

    let (reports, presses) = mpsc::channel();
    let _ = REPORT.set(reports);

    Owner::detached().run(|| {
        spawn(async move {
            if let Err(why) = run_on_main_thread(listen).await {
                eprintln!("voice: the keys' monitor did not start: {why:?}");
            }
            loop {
                while let Ok(press) = presses.try_recv() {
                    match press {
                        Press::Toggle => super::toggle(),
                        Press::Stop => super::stop(),
                    }
                }
                timer(Duration::from_millis(16)).await;
            }
        });
    });
}

/// Nothing outside a Mac: the keys are the right Option's and Escape's.
#[cfg(not(target_os = "macos"))]
pub(crate) fn watch() {}

/// Where the main thread's monitor reports a press.
#[cfg(target_os = "macos")]
static REPORT: OnceLock<Sender<Press>> = OnceLock::new();

/// Reports a press to the engine, from the main thread.
#[cfg(target_os = "macos")]
fn press(press: Press) {
    if let Some(reports) = REPORT.get() {
        let _ = reports.send(press);
    }
}

/// Installs the monitor on the main thread, where AppKit's own live.
#[cfg(target_os = "macos")]
fn listen() {
    macos::listen(press);
}

#[cfg(target_os = "macos")]
mod macos {
    use std::{
        ptr::NonNull,
        sync::atomic::{AtomicBool, Ordering},
    };

    use block2::RcBlock;
    use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags, NSEventType};

    use super::Press;

    /// The right Option's virtual key code (`NSEvent.keyCode`), the left one
    /// being 58 on every Apple keyboard.
    const RIGHT_OPTION: u16 = 61;

    /// Escape's virtual key code.
    const ESCAPE: u16 = 53;

    /// Whether the right Option is held: a press is the transition into it,
    /// not every `flagsChanged` the key takes part in.
    static HELD: AtomicBool = AtomicBool::new(false);

    /// Watches the app's own key events for a press of the right Option or of
    /// Escape. The monitor hands every event on unchanged: Escape still does
    /// what it does, and a modifier pressed alone types nothing. It is kept for
    /// the life of the app.
    pub(super) fn listen(press: fn(Press)) {
        let block = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
            let raw = event.as_ptr();
            // SAFETY: AppKit passes a live event to the monitor it calls.
            let event = unsafe { event.as_ref() };

            match event.r#type() {
                NSEventType::KeyDown if event.keyCode() == ESCAPE => press(Press::Stop),
                NSEventType::FlagsChanged => {
                    let held = event.modifierFlags().contains(NSEventModifierFlags::Option);

                    if !held {
                        HELD.store(false, Ordering::Relaxed);
                    } else if event.keyCode() == RIGHT_OPTION && !HELD.swap(true, Ordering::Relaxed)
                    {
                        press(Press::Toggle);
                    }
                }
                _ => {}
            }

            raw
        });
        // SAFETY: the block answers with the event it was given, so the app
        // goes on seeing it; the monitor outlives every window it watches.
        let mask = NSEventMask::KeyDown | NSEventMask::FlagsChanged;
        let monitor =
            unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &block) };
        std::mem::forget(monitor);
    }
}
