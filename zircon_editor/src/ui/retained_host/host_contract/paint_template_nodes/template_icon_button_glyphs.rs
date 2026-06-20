use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_icon_button_glyph_kind::icon_button_glyph_kind;
use super::template_icon_button_glyph_segments::push_icon_button_glyph_segments;
use super::template_icon_button_glyph_shapes::push_icon_button_glyph_shape;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    state: UiPainterResolvedState,
    opacity: f32,
) {
    push_icon_button_glyph_shape(
        commands,
        icon_button_glyph_kind(node),
        rect,
        clip,
        order,
        color,
        opacity,
    );

    if state == UiPainterResolvedState::Pressed {
        push_icon_button_glyph_segments(
            commands,
            rect,
            clip,
            order + 3,
            color,
            opacity * 0.28,
            &[(2.0, 13.0, 12.0, 1.0)],
        );
    }
}
