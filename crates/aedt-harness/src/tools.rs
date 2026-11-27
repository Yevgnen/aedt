//! The tools a reply may call.

use rig_harness::prelude::*;

/// The tools a reply may call: the ones that read and edit what the agent works
/// on, and one that runs commands there.
pub(crate) struct CodingTools;

impl Plugin for CodingTools {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            rig_basics::coding_tools::ReadTool,
            rig_basics::coding_tools::EditTool,
            rig_basics::coding_tools::ShellTool,
        ));
    }
}
