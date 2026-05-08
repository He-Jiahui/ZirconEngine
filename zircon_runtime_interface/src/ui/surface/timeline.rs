use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::{UiSurfaceDebugOptions, UiSurfaceDebugSnapshot};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct UiDebugTimelineFrameHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDebugTimelineFrameSummary {
    pub handle: UiDebugTimelineFrameHandle,
    pub frame_index: u64,
    pub captured_at_millis: Option<u64>,
    pub source_target_id: String,
    pub source_label: String,
    pub schema_version: u32,
    pub node_count: usize,
    pub render_command_count: usize,
    pub hit_grid_cell_count: usize,
    pub invalidation_dirty_count: usize,
    pub has_damage_region: bool,
    pub warning_count: usize,
    pub selected_node: Option<UiNodeId>,
    pub capture_options: UiSurfaceDebugOptions,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDebugTimelineRetention {
    pub capacity: usize,
    pub len: usize,
    pub first_frame: Option<UiDebugTimelineFrameHandle>,
    pub latest_frame: Option<UiDebugTimelineFrameHandle>,
    pub selected_frame: Option<UiDebugTimelineFrameHandle>,
    pub dropped_frame_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiDebugTimelineSnapshot {
    pub selected_frame: Option<UiDebugTimelineFrameHandle>,
    pub summaries: Vec<UiDebugTimelineFrameSummary>,
    pub frames: Vec<UiSurfaceDebugSnapshot>,
    pub retention: UiDebugTimelineRetention,
}
