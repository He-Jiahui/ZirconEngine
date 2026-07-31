mod frame;
mod style;
mod text;

use super::super::super::super::data::{FrameRect, HostPaneInteractionStateData, SceneNodeData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};

use self::frame::hierarchy_row_frame;
use self::style::hierarchy_row_color;
use self::text::draw_hierarchy_row_text;
use super::super::super::ACCENT;

pub(super) fn draw_hierarchy_row(
    frame: &mut HostRgbaFrame,
    viewport: &FrameRect,
    row_clip: &FrameRect,
    index: usize,
    scroll_px: f32,
    node: &SceneNodeData,
    interaction: &HostPaneInteractionStateData,
    inline_rename_value: Option<&str>,
) {
    let row = hierarchy_row_frame(viewport, index, scroll_px);
    if intersect(&row, row_clip).is_none() {
        return;
    }
    draw_rect_clipped(
        frame,
        row.clone(),
        Some(row_clip),
        hierarchy_row_color(index, node, interaction),
    );
    if node.selected {
        draw_border_clipped(frame, row.clone(), Some(row_clip), ACCENT);
    }
    draw_hierarchy_row_text(frame, &row, node, row_clip, inline_rename_value);
}
