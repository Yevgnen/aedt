//! A probe: can `rig-harness` run as a library, its frames driven by the
//! host's own loop instead of the harness's `RunnerPlugin`?
//!
//! The host here is the terminal. A real one is a window: the same seam,
//! `Wake`, is what `rig_harness::windowed`'s documentation points a window
//! plugin at.

use bevy_app::PluginsState;
use rig_harness::host::runner::RunnerPlugin;
use rig_harness::prelude::*;

/// Frames the host drove, counted by a system of its own.
#[derive(Resource, Default)]
struct Frames(u32);

fn count(mut frames: ResMut<Frames>) {
    frames.0 += 1;
}

fn main() -> AppExit {
    let mut app = App::new();
    app.set_error_handler(rig_harness::error::warn);

    // Bevy's base and the harness, minus the loop that would own the thread.
    app.add_plugins(HeadlessPlugins.build().disable::<RunnerPlugin>());
    app.add_plugins(RigHarnessPlugins);
    // A run needs a front: this one shows nothing.
    app.add_front(Front::new("spike").headless());

    // The frame this host asks for is what a wake would ask for: the wake
    // `RunnerPlugin` would have inserted is ours.
    app.insert_resource(Wake::new(|| {}));

    app.insert_resource(Frames::default());
    app.add_systems(Update, count);

    // What Bevy's own runners do before their first frame
    // (`rig_harness::host::runner` does the same, and keeps it private).
    if app.plugins_state() != PluginsState::Cleaned {
        while app.plugins_state() == PluginsState::Adding {
            bevy_tasks::tick_global_task_pools_on_main_thread();
        }
        app.finish();
        app.cleanup();
    }

    for _ in 0..3 {
        app.update();
    }

    let frames = app.world().resource::<Frames>().0;
    assert_eq!(frames, 3, "the host's frames are the app's frames");
    println!("ok: {frames} frames driven by the host");

    AppExit::Success
}
