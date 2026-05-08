use zircon_runtime_interface::ui::surface::{
    UiDebugTimelineFrameHandle, UiDebugTimelineFrameSummary, UiDebugTimelineSnapshot,
    UiSurfaceDebugSnapshot,
};

use super::model::EditorUiDebugReflectorModel;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorUiDebugTimelineModel {
    pub retention: String,
    pub selected: String,
    pub latest: String,
    pub previous_frame: Option<UiDebugTimelineFrameHandle>,
    pub next_frame: Option<UiDebugTimelineFrameHandle>,
    pub frame_rows: Vec<String>,
    pub selected_reflector: EditorUiDebugReflectorModel,
}

impl EditorUiDebugTimelineModel {
    pub(crate) fn from_timeline(timeline: &UiDebugTimelineSnapshot) -> Self {
        let selected_handle = selected_handle(timeline);
        let selected_summary =
            selected_handle.and_then(|handle| summary_for_handle(timeline, handle));
        let selected_snapshot =
            selected_handle.and_then(|handle| snapshot_for_handle(timeline, handle));
        let (previous_frame, next_frame) = selected_handle
            .map(|handle| timeline_neighbors(timeline, handle))
            .unwrap_or((None, None));

        Self {
            retention: retention_label(timeline),
            selected: selected_summary
                .map(selected_label)
                .unwrap_or_else(|| "Selected frame: none".to_string()),
            latest: timeline
                .retention
                .latest_frame
                .and_then(|handle| summary_for_handle(timeline, handle))
                .map(latest_label)
                .unwrap_or_else(|| "Latest frame: none".to_string()),
            previous_frame,
            next_frame,
            frame_rows: timeline.summaries.iter().map(frame_row).collect(),
            selected_reflector: selected_snapshot
                .map(EditorUiDebugReflectorModel::from_snapshot)
                .unwrap_or_else(EditorUiDebugReflectorModel::no_active_surface),
        }
    }
}

fn selected_handle(timeline: &UiDebugTimelineSnapshot) -> Option<UiDebugTimelineFrameHandle> {
    timeline
        .selected_frame
        .filter(|handle| summary_for_handle(timeline, *handle).is_some())
        .or(timeline.retention.latest_frame)
}

fn summary_for_handle(
    timeline: &UiDebugTimelineSnapshot,
    handle: UiDebugTimelineFrameHandle,
) -> Option<&UiDebugTimelineFrameSummary> {
    timeline
        .summaries
        .iter()
        .find(|summary| summary.handle == handle)
}

fn snapshot_for_handle(
    timeline: &UiDebugTimelineSnapshot,
    handle: UiDebugTimelineFrameHandle,
) -> Option<&UiSurfaceDebugSnapshot> {
    timeline
        .summaries
        .iter()
        .position(|summary| summary.handle == handle)
        .and_then(|index| timeline.frames.get(index))
}

fn timeline_neighbors(
    timeline: &UiDebugTimelineSnapshot,
    handle: UiDebugTimelineFrameHandle,
) -> (
    Option<UiDebugTimelineFrameHandle>,
    Option<UiDebugTimelineFrameHandle>,
) {
    let Some(index) = timeline
        .summaries
        .iter()
        .position(|summary| summary.handle == handle)
    else {
        return (None, None);
    };
    let previous = index
        .checked_sub(1)
        .and_then(|previous| timeline.summaries.get(previous))
        .map(|summary| summary.handle);
    let next = timeline
        .summaries
        .get(index + 1)
        .map(|summary| summary.handle);
    (previous, next)
}

fn retention_label(timeline: &UiDebugTimelineSnapshot) -> String {
    format!(
        "Timeline: {}/{} frames retained, dropped {}, first={}, latest={}, selected={}",
        timeline.retention.len,
        timeline.retention.capacity,
        timeline.retention.dropped_frame_count,
        handle_label(timeline.retention.first_frame),
        handle_label(timeline.retention.latest_frame),
        handle_label(timeline.retention.selected_frame),
    )
}

fn selected_label(summary: &UiDebugTimelineFrameSummary) -> String {
    format!(
        "Selected frame: handle={} frame={} source={} nodes={} commands={}",
        summary.handle.0,
        summary.frame_index,
        summary.source_label,
        summary.node_count,
        summary.render_command_count,
    )
}

fn latest_label(summary: &UiDebugTimelineFrameSummary) -> String {
    format!(
        "Latest frame: handle={} frame={} source={}",
        summary.handle.0, summary.frame_index, summary.source_label,
    )
}

fn frame_row(summary: &UiDebugTimelineFrameSummary) -> String {
    format!(
        "handle={} frame={} source={} nodes={} commands={} hit_cells={} dirty={} warnings={} selected_node={:?}",
        summary.handle.0,
        summary.frame_index,
        summary.source_label,
        summary.node_count,
        summary.render_command_count,
        summary.hit_grid_cell_count,
        summary.invalidation_dirty_count,
        summary.warning_count,
        summary.selected_node.map(|node| node.0),
    )
}

fn handle_label(handle: Option<UiDebugTimelineFrameHandle>) -> String {
    handle
        .map(|handle| handle.0.to_string())
        .unwrap_or_else(|| "none".to_string())
}
