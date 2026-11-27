//! The reader's own key for the microphone: the right Option, pressed on its
//! own.
//!
//! AppKit reports a modifier key only as `flagsChanged`, and only to the
//! monitor that runs before the responder chain, so the press comes from a
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

/// Calls `toggle` on the engine at each press of the shortcut, once for the
/// app.
#[cfg(target_os = "macos")]
pub(crate) fn watch(toggle: fn()) {
    static WATCHING: OnceLock<()> = OnceLock::new();
    if WATCHING.set(()).is_err() {
        return;
    }

    let (reports, presses) = mpsc::channel();
    let _ = REPORT.set(reports);

    Owner::detached().run(|| {
        spawn(async move {
            if let Err(why) = run_on_main_thread(listen).await {
                eprintln!("voice: the shortcut's monitor did not start: {why:?}");
            }
            loop {
                while presses.try_recv().is_ok() {
                    toggle();
                }
                timer(Duration::from_millis(16)).await;
            }
        });
    });
}

/// Nothing outside a Mac: the shortcut is the right Option's.
#[cfg(not(target_os = "macos"))]
pub(crate) fn watch(_toggle: fn()) {}

/// Where the main thread's monitor reports a press.
#[cfg(target_os = "macos")]
static REPORT: OnceLock<Sender<()>> = OnceLock::new();

/// Reports a press to the engine, from the main thread.
#[cfg(target_os = "macos")]
fn press() {
    if let Some(reports) = REPORT.get() {
        let _ = reports.send(());
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
    use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags};

    /// The right Option's virtual key code (`NSEvent.keyCode`), the left one
    /// being 58 on every Apple keyboard.
    const RIGHT_OPTION: u16 = 61;

    /// Whether the right Option is held: a press is the transition into it,
    /// not every `flagsChanged` the key takes part in.
    static HELD: AtomicBool = AtomicBool::new(false);

    /// Watches the app's own key events for a press of the right Option. The
    /// monitor hands the event on unchanged, as a modifier pressed alone types
    /// nothing, and is kept for the life of the app.
    pub(super) fn listen(press: fn()) {
        let block = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
            let raw = event.as_ptr();
            // SAFETY: AppKit passes a live event to the monitor it calls.
            let event = unsafe { event.as_ref() };
            let held = event.modifierFlags().contains(NSEventModifierFlags::Option);

            if !held {
                HELD.store(false, Ordering::Relaxed);
            } else if event.keyCode() == RIGHT_OPTION && !HELD.swap(true, Ordering::Relaxed) {
                press();
            }

            raw
        });
        // SAFETY: the block answers with the event it was given, so the app
        // goes on seeing it; the monitor outlives every window it watches.
        let monitor = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::FlagsChanged, &block)
        };
        std::mem::forget(monitor);
    }
}
