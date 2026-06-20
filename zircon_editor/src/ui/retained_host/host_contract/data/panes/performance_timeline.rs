use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct PerformanceTimelineFrameRowData {
    pub stream: SharedString,
    pub name: SharedString,
    pub frame_index: u64,
    pub duration_label: SharedString,
    pub budget_label: SharedString,
    pub budget_usage_label: SharedString,
    pub duration_ratio: f32,
    pub bar_fill_ratio: f32,
    pub budget_marker_ratio: f32,
    pub over_budget: bool,
}

#[derive(Clone, Default)]
pub(crate) struct PerformanceTimelineSpanRowData {
    pub stream: SharedString,
    pub category: SharedString,
    pub name: SharedString,
    pub path: SharedString,
    pub duration_label: SharedString,
    pub depth: u16,
}

#[derive(Clone, Default)]
pub(crate) struct PerformanceTimelineHotspotRowData {
    pub stream: SharedString,
    pub category: SharedString,
    pub name: SharedString,
    pub path: SharedString,
    pub total_label: SharedString,
    pub average_label: SharedString,
    pub count_label: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct PerformanceTimelineCaptureControlData {
    pub label: SharedString,
    pub action_id: SharedString,
    pub enabled: bool,
}

#[derive(Clone, Default)]
pub(crate) struct PerformanceTimelinePaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub frame_rows: ModelRc<PerformanceTimelineFrameRowData>,
    pub span_rows: ModelRc<PerformanceTimelineSpanRowData>,
    pub hotspot_rows: ModelRc<PerformanceTimelineHotspotRowData>,
    pub capture_controls: ModelRc<PerformanceTimelineCaptureControlData>,
    pub summary: SharedString,
    pub session_label: SharedString,
    pub output_label: SharedString,
}
