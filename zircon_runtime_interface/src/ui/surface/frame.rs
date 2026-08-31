use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ui::pipeline::UiPipelineFrameReport;

use super::{
    UiFocusPath, UiFocusState, UiHitTestGrid, UiRenderFrameExtract, UiSurfaceRebuildDebugStats,
};
use crate::ui::event_ui::UiTreeId;
use crate::ui::layout::UiLayoutEngineSelectionReport;
use crate::ui::window::{UiWindowMetrics, UiWindowPixelPosition, UiWindowRedrawReason};

use super::UiArrangedTree;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceFrame {
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    pub domain_generations: UiSurfaceFrameDomainGenerations,
    pub tree_id: UiTreeId,
    #[serde(default)]
    pub window_state: UiSurfaceWindowState,
    pub arranged_tree: Arc<UiArrangedTree>,
    pub render_extract: Arc<UiRenderFrameExtract>,
    pub hit_grid: Arc<UiHitTestGrid>,
    pub focus_state: Arc<UiFocusState>,
    #[serde(default)]
    pub focus_path: Arc<UiFocusPath>,
    pub last_rebuild: UiSurfaceRebuildDebugStats,
    #[serde(default)]
    pub layout_engine_report: Arc<UiLayoutEngineSelectionReport>,
    #[serde(default)]
    pub pipeline_report: Arc<UiPipelineFrameReport>,
}

/// Independent immutable-domain generations inside one atomically published surface frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSurfaceFrameDomainGenerations {
    pub layout: u64,
    pub render: u64,
    pub hit_test: u64,
    pub focus: u64,
    pub pipeline: u64,
    pub window: u64,
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
