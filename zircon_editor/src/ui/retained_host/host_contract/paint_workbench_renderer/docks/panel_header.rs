use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{is_visible_frame, translated};
use super::super::super::paint_primitives::{draw_rect, draw_separator_line};
use super::super::super::paint_template_nodes::draw_template_nodes;
use super::super::{SEPARATOR, TOP_BAR};

pub(in crate::ui::retained_host::host_contract) fn draw_panel_header(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    header_frame: &FrameRect,
) {
    let header = translated(header_frame, origin.x, origin.y);
    if !is_visible_frame(&header) {
        return;
    }
    draw_rect(frame, header.clone(), TOP_BAR);
    {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "painter_panel_header_template_nodes"
        );
        draw_template_nodes(frame, nodes, origin, &header, None);
    }
    draw_separator_line(
        frame,
        header.x.max(0.0) as u32,
        (header.y + header.height - 1.0).max(0.0) as u32,
        header.width.max(0.0) as u32,
        SEPARATOR,
    );
}
