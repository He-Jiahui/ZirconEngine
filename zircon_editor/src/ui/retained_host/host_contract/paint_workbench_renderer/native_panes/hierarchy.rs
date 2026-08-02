mod row;
mod viewport;

use crate::ui::retained_host::app::hierarchy_rename::{
    HIERARCHY_INLINE_RENAME_CONTROL_ID, hierarchy_inline_rename_target_id,
};
use crate::ui::retained_host::hierarchy_pointer::{
    HierarchyRowMetrics, current_hierarchy_row_metrics, hierarchy_row_metrics_from_host_metrics,
};

pub(super) use viewport::hierarchy_viewport_frame;

use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, PaneData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use row::draw_hierarchy_row;

pub(in crate::ui::retained_host::host_contract) fn draw_hierarchy_rows(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    let node_count = pane.hierarchy.hierarchy_nodes.row_count();
    if node_count == 0 {
        return false;
    }
    let viewport = hierarchy_viewport_frame(pane, body);
    let Some(row_clip) = intersect(&viewport, clip) else {
        return false;
    };
    let scroll_px = interaction.hierarchy_scroll_px.max(0.0);
    let row_metrics = current_hierarchy_row_metrics();

    for index in
        visible_hierarchy_row_range(&viewport, &row_clip, scroll_px, node_count, row_metrics)
    {
        let Some(node) = pane.hierarchy.hierarchy_nodes.row_data(index) else {
            continue;
        };
        let inline_rename_value = inline_hierarchy_rename_value(&node, text_input_focus);
        draw_hierarchy_row(
            frame,
            &viewport,
            &row_clip,
            index,
            scroll_px,
            &node,
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
mod tests {
    use super::*;

    #[test]
    fn inline_rename_focus_only_replaces_the_matching_row_text() {
        let focus = HostTextInputFocusData {
            control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
            dispatch_kind: "hierarchy_inline_rename:7".into(),
            value_text: "Renamed".into(),
            ..HostTextInputFocusData::default()
        };
        let selected = SceneNodeData {
            id: "7".into(),
            selected: true,
            ..SceneNodeData::default()
        };
        let other_selected = SceneNodeData {
            id: "8".into(),
            selected: true,
            ..SceneNodeData::default()
        };

        assert_eq!(
            inline_hierarchy_rename_value(&selected, Some(&focus)),
            Some("Renamed")
        );
        assert_eq!(
            inline_hierarchy_rename_value(&other_selected, Some(&focus)),
            None
        );
    }

    #[test]
    fn visible_row_range_is_bounded_by_the_clipped_viewport() {
        let viewport = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 100.0,
        };
        let metrics = hierarchy_row_metrics_from_host_metrics(
            crate::ui::retained_host::host_contract::paint_theme::METRICS,
        );
        let range = visible_hierarchy_row_range(&viewport, &viewport, 560.0, 10_000, metrics);
        let maximum_rows =
            (viewport.height / (metrics.row_height + metrics.row_gap)).ceil() as usize + 2;

        assert!(range.start > 0);
        assert!(range.end < 10_000);
        assert!(range.len() <= maximum_rows);
    }

    #[test]
    fn visible_row_range_is_empty_for_an_empty_clip() {
        let viewport = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 100.0,
        };
        let empty_clip = FrameRect {
            height: 0.0,
            ..viewport.clone()
        };

        assert!(
            visible_hierarchy_row_range(
                &viewport,
                &empty_clip,
                0.0,
                10_000,
                hierarchy_row_metrics_from_host_metrics(
                    crate::ui::retained_host::host_contract::paint_theme::METRICS,
                ),
            )
            .is_empty()
        );
    }
}
