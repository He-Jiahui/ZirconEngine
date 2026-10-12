use super::super::super::data::FrameRect;
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::metrics::template_node_text_line_height;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_text_command(
    commands: &mut Vec<HostPaintCommand>,
    text_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: String,
    color: [u8; 4],
    font_size: f32,
    text_style: UiTextRunPaintStyle,
    opacity: f32,
) {
    if !is_paintable_text_slot(text_rect, clip, font_size) || label.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::wrapped_text(
        FrameRect {
            x: text_rect.x,
            y: text_rect.y,
            width: text_rect.width,
            height: text_rect.height,
        },
        Some(clip.clone()),
        order,
        label,
        color,
        font_size,
        template_node_text_line_height(font_size),
        text_style,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_paintable_text_slot(
    text_rect: &FrameRect,
    clip: &FrameRect,
    font_size: f32,
) -> bool {
    is_paintable_text_rect(text_rect)
        && is_paintable_font(font_size)
        && intersect(text_rect, clip).is_some()
}

fn is_paintable_text_rect(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

fn is_paintable_font(font_size: f32) -> bool {
    font_size.is_finite() && font_size > 0.0
}

#[cfg(test)]
#[path = "tests/command.rs"]
mod tests;
