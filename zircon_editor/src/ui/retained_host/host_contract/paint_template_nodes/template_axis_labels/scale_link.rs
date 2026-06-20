use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::style::scale_link_color;

const LINK_LOBE_WIDTH: f32 = 6.0;
const LINK_LOBE_HEIGHT: f32 = 7.0;
const LINK_OVERLAP: f32 = 2.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_scale_link(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = scale_link_color(node);
    let (start_x, start_y) = scale_link_origin(node, rect);
    for lobe in [
        FrameRect {
            x: start_x,
            y: start_y,
            width: LINK_LOBE_WIDTH,
            height: LINK_LOBE_HEIGHT,
        },
        FrameRect {
            x: start_x + LINK_LOBE_WIDTH - LINK_OVERLAP,
            y: start_y,
            width: LINK_LOBE_WIDTH,
            height: LINK_LOBE_HEIGHT,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            lobe,
            Some(clip.clone()),
            order,
            None,
            Some(color),
            1.0,
            3.0,
            opacity,
        ));
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: start_x + LINK_LOBE_WIDTH - LINK_OVERLAP + 1.0,
            y: start_y + LINK_LOBE_HEIGHT * 0.5,
            width: LINK_OVERLAP,
            height: 1.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn scale_link_origin(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> (f32, f32) {
    let total_width = LINK_LOBE_WIDTH * 2.0 - LINK_OVERLAP;
    (
        rect.x + (rect.width - total_width).max(0.0) * 0.5 + node.layout_offset_x,
        rect.y + (rect.height - LINK_LOBE_HEIGHT).max(0.0) * 0.5 + node.layout_offset_y,
    )
}
