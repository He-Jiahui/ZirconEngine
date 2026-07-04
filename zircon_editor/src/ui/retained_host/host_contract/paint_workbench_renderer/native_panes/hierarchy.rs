mod row;
mod viewport;

pub(super) use viewport::hierarchy_viewport_frame;

use super::super::super::data::{FrameRect, HostPaneInteractionStateData, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use row::draw_hierarchy_row;

pub(in crate::ui::retained_host::host_contract) fn draw_hierarchy_rows(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
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

    for index in 0..node_count {
        let Some(node) = pane.hierarchy.hierarchy_nodes.row_data(index) else {
            continue;
        };
        draw_hierarchy_row(
            frame,
            &viewport,
            &row_clip,
            index,
            scroll_px,
            &node,
            interaction,
        );
    }
    true
}
