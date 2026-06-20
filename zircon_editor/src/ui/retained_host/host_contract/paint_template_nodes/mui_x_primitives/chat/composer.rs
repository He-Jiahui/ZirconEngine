use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chat_composer(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::super::node_radius(node).max(rect.height * 0.5);
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::super::node_background(node).unwrap_or(PALETTE.surface_inset),
        1.0,
        radius,
        opacity,
    );
    super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width - rect.height + 4.0,
            y: rect.y + 4.0,
            width: (rect.height - 8.0).max(1.0),
            height: (rect.height - 8.0).max(1.0),
        },
        clip,
        order + 1,
        PALETTE.accent,
        0.0,
        rect.height,
        opacity,
    );
}
