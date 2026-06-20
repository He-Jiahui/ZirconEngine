use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::{
    MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION, MUI_X_CHAT_INSET, MUI_X_CHAT_STREAMING_HEIGHT,
};
use super::style::chat_surface_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_agent_chat(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::super::node_radius(node).max(8.0);
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::super::node_background(node).unwrap_or_else(|| chat_surface_color(node)),
        0.0,
        radius,
        opacity,
    );

    let bubble_height = (rect.height * MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION).max(8.0);
    super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + MUI_X_CHAT_INSET,
            y: rect.y + MUI_X_CHAT_INSET,
            width: rect.width * 0.58,
            height: bubble_height,
        },
        clip,
        order + 1,
        PALETTE.surface,
        0.0,
        5.0,
        opacity,
    );
    super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.36,
            y: rect.y + MUI_X_CHAT_INSET + bubble_height + 3.0,
            width: (rect.width * 0.58 - MUI_X_CHAT_INSET).max(1.0),
            height: bubble_height,
        },
        clip,
        order + 2,
        PALETTE.surface_selected,
        0.0,
        5.0,
        opacity,
    );

    if node.component_variant.as_str().contains("streaming") || node.popup_open {
        super::super::push_quad(
            commands,
            FrameRect {
                x: rect.x + MUI_X_CHAT_INSET,
                y: rect.y + rect.height - MUI_X_CHAT_INSET,
                width: (rect.width * 0.42).max(1.0),
                height: MUI_X_CHAT_STREAMING_HEIGHT,
            },
            clip,
            order + 3,
            PALETTE.accent,
            0.0,
            MUI_X_CHAT_STREAMING_HEIGHT * 0.5,
            opacity,
        );
    }
}
