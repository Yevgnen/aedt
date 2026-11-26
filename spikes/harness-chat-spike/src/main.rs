//! A probe: the rig harness as a library, chatting with a real model through
//! its own headless front, with its file tools and their results.
//!
//! It is not the `rig` launcher: this program builds the app itself, adds the
//! plugins it wants, and runs it. `-p` asks one prompt and exits, as `rig -p`
//! does; `--json` writes the run as JSON lines, tool calls and their outputs
//! included; `--model vendor/model` names the model, whose key comes from the
//! environment.
//!
//! ```text
//! RIG_HOME=… harness-chat-spike --model deepseek/deepseek-flash \
//!     -p "Read Cargo.toml with the read tool and tell me the package name."
//! ```

use rig_harness::{App, AppExit, HeadlessPlugins, RigHarnessPlugins};

fn main() -> AppExit {
    let mut app = App::new();
    // A failing system, observer or command from a plugin is logged.
    app.set_error_handler(rig_harness::error::warn);
    app.add_plugins((
        // Bevy's base, the agent loop's clock, the signals and the session.
        HeadlessPlugins,
        RigHarnessPlugins,
        // What to say, which model, and the tools a reply may call.
        rig_basics::prompt::DefaultPromptPlugin,
        rig_basics::models::ModelsPlugin,
        rig_basics::coding_tools::ReadTool,
        rig_basics::coding_tools::ShellTool,
        // The front: `-p`/`--json`, which is the front of every run that is
        // not on an interactive terminal.
        rig_basics::print::PrintPlugin,
    ));
    rig_harness::run(app)
}
