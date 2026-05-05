use serde::{Deserialize, Serialize};

use super::{UiFocusState, UiHitTestGrid, UiRenderExtract, UiSurfaceRebuildDebugStats};
use crate::ui::event_ui::UiTreeId;

use super::UiArrangedTree;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceFrame {
    pub tree_id: UiTreeId,
    pub arranged_tree: UiArrangedTree,
    pub render_extract: UiRenderExtract,
    pub hit_grid: UiHitTestGrid,
    pub focus_state: UiFocusState,
    pub last_rebuild: UiSurfaceRebuildDebugStats,
}
