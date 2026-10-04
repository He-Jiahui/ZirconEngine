use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::{frame_from_template, translated};

pub(super) fn asset_tree_row_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    body: &FrameRect,
    hovered_index: usize,
    scroll_px: f32,
) -> Option<FrameRect> {
    let row = nodes
        .metadata::<AssetContentPaintMetadata>()?
        .activity_tree_node_row(hovered_index)?;
    let node = nodes.get(row)?;
    let mut frame = translated(&frame_from_template(&node.frame), body.x, body.y);
    frame.y -= scroll_px;
    Some(frame)
}

#[cfg(test)]
#[path = "tests/frame.rs"]
mod tests;
