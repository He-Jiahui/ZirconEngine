use serde::{Deserialize, Serialize};

use crate::ui::ecs::UiEcsProjectionSnapshot;
use crate::ui::pipeline::UiPipelineFrameReport;

use super::{
    UiFocusPath, UiFocusState, UiHitTestGrid, UiRenderExtract, UiSurfaceRebuildDebugStats,
};
use crate::ui::event_ui::UiTreeId;
use crate::ui::layout::UiLayoutEngineSelectionReport;
use crate::ui::window::{UiWindowMetrics, UiWindowPixelPosition, UiWindowRedrawReason};

use super::UiArrangedTree;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceFrame {
    pub tree_id: UiTreeId,
    #[serde(default)]
    pub window_state: UiSurfaceWindowState,
    pub arranged_tree: UiArrangedTree,
    pub render_extract: UiRenderExtract,
    pub hit_grid: UiHitTestGrid,
    pub focus_state: UiFocusState,
    #[serde(default)]
    pub focus_path: UiFocusPath,
    pub last_rebuild: UiSurfaceRebuildDebugStats,
    #[serde(default)]
    pub layout_engine_report: UiLayoutEngineSelectionReport,
    #[serde(default)]
    pub pipeline_report: UiPipelineFrameReport,
    #[serde(default)]
    pub ecs_projection: UiEcsProjectionSnapshot,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSurfaceWindowState {
    pub metrics: Option<UiWindowMetrics>,
    pub position: Option<UiWindowPixelPosition>,
    pub focused: Option<bool>,
    pub application_active: Option<bool>,
    pub occluded: Option<bool>,
    pub close_requested: bool,
    pub closed: bool,
    pub destroyed: bool,
    pub redraw_requested: bool,
    pub redraw_request_count: u64,
    pub last_redraw_reason: Option<UiWindowRedrawReason>,
}
