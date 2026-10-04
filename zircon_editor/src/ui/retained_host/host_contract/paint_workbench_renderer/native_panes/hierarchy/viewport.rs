use crate::ui::retained_host::hierarchy_pointer::HierarchyPaintMetadata;

use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_geometry::{
    frame_from_template, is_visible_frame, translated,
};

pub(in crate::ui::retained_host::host_contract) fn hierarchy_viewport_frame(
    pane: &PaneData,
    body: &FrameRect,
) -> FrameRect {
    let nodes = &pane.hierarchy.nodes;
    nodes
        .metadata::<HierarchyPaintMetadata>()
        .into_iter()
        .flat_map(HierarchyPaintMetadata::viewport_node_rows)
        .filter_map(|&row| nodes.get(row))
        .map(|node| translated(&frame_from_template(&node.frame), body.x, body.y))
        .find(is_visible_frame)
        .unwrap_or_else(|| body.clone())
}

#[cfg(test)]
#[path = "tests/viewport.rs"]
mod tests;
