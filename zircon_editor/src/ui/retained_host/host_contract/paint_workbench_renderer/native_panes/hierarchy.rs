mod row;
mod viewport;

use crate::ui::retained_host::app::hierarchy_rename::{
    hierarchy_inline_rename_target_id, HIERARCHY_INLINE_RENAME_CONTROL_ID,
};
use crate::ui::retained_host::hierarchy_pointer::{
    hierarchy_row_metrics_from_host_metrics, HierarchyRowMetrics,
};

pub(in crate::ui::retained_host::host_contract) use row::hierarchy_row_frame;
pub(in crate::ui::retained_host::host_contract) use viewport::hierarchy_viewport_frame;

use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, PaneData, SceneNodeData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use row::draw_hierarchy_row;

pub(in crate::ui::retained_host::host_contract) fn draw_hierarchy_rows(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    viewport: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    text_input_focus: Option<&HostTextInputFocusData>,
    row_metrics: HierarchyRowMetrics,
) -> bool {
    let node_count = pane.hierarchy.hierarchy_nodes.row_count();
    if node_count == 0 {
        return false;
    }
    let Some(row_clip) = intersect(viewport, clip) else {
        return false;
    };
    let scroll_px = interaction.hierarchy_scroll_px.max(0.0);

    for index in
        visible_hierarchy_row_range(viewport, &row_clip, scroll_px, node_count, row_metrics)
    {
        let Some(node) = pane.hierarchy.hierarchy_nodes.get(index) else {
            continue;
        };
        let inline_rename_value = inline_hierarchy_rename_value(node, text_input_focus);
        draw_hierarchy_row(
            frame,
            viewport,
            &row_clip,
            index,
            scroll_px,
            row_metrics,
            node,
            interaction,
            inline_rename_value,
        );
    }
    true
}

fn inline_hierarchy_rename_value<'a>(
    node: &SceneNodeData,
    text_input_focus: Option<&'a HostTextInputFocusData>,
) -> Option<&'a str> {
    text_input_focus
        .filter(|focus| focus.control_id.as_str() == HIERARCHY_INLINE_RENAME_CONTROL_ID)
        .filter(|focus| {
            hierarchy_inline_rename_target_id(focus.dispatch_kind.as_str())
                .is_some_and(|node_id| node.id.as_str() == node_id)
        })
        .map(|focus| focus.value_text.as_str())
}

fn visible_hierarchy_row_range(
    viewport: &FrameRect,
    row_clip: &FrameRect,
    scroll_px: f32,
    node_count: usize,
    row_metrics: HierarchyRowMetrics,
) -> std::ops::Range<usize> {
    if node_count == 0
        || row_clip.height <= 0.0
        || !viewport.y.is_finite()
        || !row_clip.y.is_finite()
        || !row_clip.height.is_finite()
        || !scroll_px.is_finite()
    {
        return 0..0;
    }

    let row_pitch = row_metrics.row_height + row_metrics.row_gap;
    let first_row_y = viewport.y + row_metrics.row_y;
    let scroll_px = scroll_px.max(0.0);
    let start = ((row_clip.y + scroll_px - first_row_y - row_metrics.row_height) / row_pitch)
        .floor()
        .max(0.0) as usize;
    let end = ((row_clip.y + row_clip.height + scroll_px - first_row_y) / row_pitch)
        .ceil()
        .max(0.0) as usize;
    let start = start.min(node_count);
    start..end.max(start).min(node_count)
}

#[cfg(test)]
#[path = "tests/hierarchy.rs"]
mod tests;
