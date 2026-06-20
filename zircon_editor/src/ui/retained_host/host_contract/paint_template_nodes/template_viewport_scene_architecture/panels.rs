use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style::border_color;
use super::super::template_viewport_scene_structure::push_base_surface;
use super::primitives::color_with_alpha_factor;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_side_panel_detail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    let line_color = color_with_alpha_factor(border_color(node), 1.75);
    for y in [rect.y + 36.0, rect.y + 78.0, rect.y + 126.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + 12.0,
                y,
                width: (rect.width - 24.0).max(1.0),
                height: 1.0,
            },
            Some(clip.clone()),
            order + 1,
            Some(line_color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}
