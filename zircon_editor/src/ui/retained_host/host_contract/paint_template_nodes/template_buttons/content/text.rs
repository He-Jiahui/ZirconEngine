use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::{BUTTON_FONT_SIZE, BUTTON_LINE_HEIGHT};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_button_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    x: f32,
    width: f32,
    label: String,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        FrameRect {
            x,
            y: rect.y + (rect.height - BUTTON_LINE_HEIGHT).max(0.0) * 0.5,
            width,
            height: BUTTON_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        color,
        if node.font_size.is_finite() && node.font_size > 0.0 {
            node.font_size.min(rect.height.max(1.0))
        } else {
            BUTTON_FONT_SIZE
        },
        BUTTON_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
