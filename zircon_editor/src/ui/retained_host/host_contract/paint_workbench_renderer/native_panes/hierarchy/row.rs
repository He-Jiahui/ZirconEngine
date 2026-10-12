mod frame;
mod style;
mod text;

use crate::ui::retained_host::hierarchy_pointer::HierarchyRowMetrics;

use super::super::super::super::data::{FrameRect, HostPaneInteractionStateData, SceneNodeData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};

pub(in crate::ui::retained_host::host_contract) use self::frame::hierarchy_row_frame;
use self::style::hierarchy_row_color;
use self::text::draw_hierarchy_row_text;
use super::super::super::ACCENT;

pub(super) fn draw_hierarchy_row(
    frame: &mut HostRgbaFrame,
    viewport: &FrameRect,
    row_clip: &FrameRect,
    index: usize,
    scroll_px: f32,
    row_metrics: HierarchyRowMetrics,
    node: &SceneNodeData,
    interaction: &HostPaneInteractionStateData,
    inline_rename_value: Option<&str>,
) {
    let row = hierarchy_row_frame(viewport, index, scroll_px, row_metrics);
    if intersect(&row, row_clip).is_none() {
        return;
    }
    let command_start = frame
        .recording
        .as_ref()
        .map(|recording| recording.command_count());
    draw_rect_clipped(
        frame,
        row.clone(),
        Some(row_clip),
        hierarchy_row_color(index, node, interaction),
    );
    if node.selected
        && interaction.focused_template_control_id == "HierarchyListPanel"
        && interaction.template_focus_visible
    {
        draw_border_clipped(frame, row.clone(), Some(row_clip), ACCENT);
    }
    draw_hierarchy_row_text(frame, &row, node, row_clip, inline_rename_value);
    if let (Some(start), Some(recording)) = (command_start, frame.recording.as_ref()) {
        crate::ui::retained_host::host_contract::paint_template_nodes::record_native_hierarchy_row(
            node,
            index,
            &row,
            row_clip,
            recording.commands_since(start),
        );
    }
}
