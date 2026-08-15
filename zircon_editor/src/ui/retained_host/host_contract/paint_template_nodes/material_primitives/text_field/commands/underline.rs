use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::{field_stroke_color, field_stroke_width, MUI_FIELD_STANDARD_UNDERLINE};

pub(super) fn push_underline(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let thickness = field_stroke_width(node)
        .max(MUI_FIELD_STANDARD_UNDERLINE)
        .min(rect.height.max(1.0));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y + rect.height - thickness,
            width: rect.width,
            height: thickness,
        },
        Some(clip.clone()),
        order,
        Some(field_stroke_color(node)),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
