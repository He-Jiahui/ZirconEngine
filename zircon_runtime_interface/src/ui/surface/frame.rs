use serde::{Deserialize, Serialize};

use crate::ui::ecs::UiEcsProjectionSnapshot;
use crate::ui::pipeline::UiPipelineFrameReport;

use super::{UiFocusState, UiHitTestGrid, UiRenderExtract, UiSurfaceRebuildDebugStats};
use crate::ui::event_ui::UiTreeId;
use crate::ui::layout::UiLayoutEngineSelectionReport;

use super::UiArrangedTree;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceFrame {
    pub tree_id: UiTreeId,
    pub arranged_tree: UiArrangedTree,
    pub render_extract: UiRenderExtract,
    pub hit_grid: UiHitTestGrid,
    pub focus_state: UiFocusState,
    pub last_rebuild: UiSurfaceRebuildDebugStats,
    #[serde(default)]
    pub layout_engine_report: UiLayoutEngineSelectionReport,
    #[serde(default)]
    pub pipeline_report: UiPipelineFrameReport,
    #[serde(default)]
    pub ecs_projection: UiEcsProjectionSnapshot,
}
