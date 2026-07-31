use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::frame_is_within;
use super::layout::content_centered_y;
use super::metrics::button_label_line_height;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_button_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    x: f32,
    y_offset: f32,
    width: f32,
    font_size: f32,
    text_style: UiTextRunPaintStyle,
    label: String,
    color: [u8; 4],
    opacity: f32,
) {
    let line_height = button_label_line_height(font_size);
    let text_rect = FrameRect {
        x,
        y: content_centered_y(rect, line_height) + y_offset,
        width,
        height: line_height,
    };
    if !frame_is_within(&text_rect, rect) {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label,
        color,
        font_size,
        line_height,
        text_style,
        opacity,
    ));
}
