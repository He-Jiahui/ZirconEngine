use crate::core::gateway::EditorRuntimeHighlightSet;
use crate::scene::viewport::DisplayMode;
use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::SceneViewportController;

const SHADED_HIGHLIGHT_TINT: [f32; 4] = [1.0, 0.76, 0.18, 1.0];
const WIRE_HIGHLIGHT_TINT: [f32; 4] = [1.0, 0.88, 0.32, 1.0];

impl SceneViewportController {
    /// Projects active-domain authoring state into the runtime-neutral overlay contract.
    pub(crate) fn build_runtime_highlight_set(&self) -> EditorRuntimeHighlightSet {
        let selection = &self.state.selection;
        EditorRuntimeHighlightSet::new(
            self.runtime_viewport(),
            selection.revision(),
            selection.active_items().iter().copied(),
            true,
            self.highlight_tint(),
        )
    }

    fn runtime_viewport(&self) -> ZrRuntimeViewportHandle {
        self.state.viewport.runtime_viewport()
    }

    fn highlight_tint(&self) -> [f32; 4] {
        match self.state.settings.display_mode {
            DisplayMode::WireOnly => WIRE_HIGHLIGHT_TINT,
            DisplayMode::Shaded | DisplayMode::WireOverlay => SHADED_HIGHLIGHT_TINT,
        }
    }
}
