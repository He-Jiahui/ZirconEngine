use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::{UiSurfaceDebugOptions, UiSurfaceDebugSnapshot};

/// 单次捕获的选择句柄；保留区淘汰旧帧时，现存帧仍按句柄定位。
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct UiDebugTimelineFrameHandle(pub u64);

/// 单帧轻量摘要，供 Editor 时间线列表展示而无需展开完整快照。
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

/// 同步报告保留容量、当前区间、选择和累计淘汰数。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDebugTimelineRetention {
    pub capacity: usize,
    pub len: usize,
    pub first_frame: Option<UiDebugTimelineFrameHandle>,
    pub latest_frame: Option<UiDebugTimelineFrameHandle>,
    pub selected_frame: Option<UiDebugTimelineFrameHandle>,
    pub dropped_frame_count: u64,
}

/// 时间线传输快照；`summaries` 与 `frames` 按同一保留队列的顺序对应。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiDebugTimelineSnapshot {
    pub selected_frame: Option<UiDebugTimelineFrameHandle>,
    pub summaries: Vec<UiDebugTimelineFrameSummary>,
    pub frames: Vec<UiSurfaceDebugSnapshot>,
    pub retention: UiDebugTimelineRetention,
}
