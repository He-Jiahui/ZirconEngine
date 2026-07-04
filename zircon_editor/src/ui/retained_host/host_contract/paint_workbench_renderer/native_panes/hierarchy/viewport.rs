use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_geometry::{
    frame_from_template, is_visible_frame, translated,
};

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::native_panes) fn hierarchy_viewport_frame(
    pane: &PaneData,
    body: &FrameRect,
) -> FrameRect {
    (0..pane.hierarchy.nodes.row_count())
        .filter_map(|row| pane.hierarchy.nodes.row_data(row))
        .find_map(|node| {
            matches!(
                node.control_id.as_str(),
                "HierarchyListPanel" | "HierarchyTreeSlotAnchor"
            )
            .then(|| translated(&frame_from_template(&node.frame), body.x, body.y))
            .filter(is_visible_frame)
        })
        .unwrap_or_else(|| body.clone())
}
