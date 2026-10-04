//! 内容编排的文字出口；完整行框放不下时省略文字，裁剪框仍用于最终像素覆盖。

use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::frame_is_within;
use super::layout::content_centered_y;
use super::metrics::button_label_line_height;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

/// 调用方已给出内容槽及测量风格；完整行框越出按钮时不输出，避免窄或矮控件产生外溢字形。
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
