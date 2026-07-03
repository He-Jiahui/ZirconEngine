use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::draw_text_with_size_and_style_impl;

pub(in crate::ui::retained_host::host_contract::paint_text) const DEFAULT_FONT_SIZE: f32 = 12.0;
pub(in crate::ui::retained_host::host_contract::paint_text) const DEFAULT_LINE_HEIGHT: f32 = 14.0;

pub(in crate::ui::retained_host::host_contract) fn draw_text(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    draw_text_with_size(
        frame,
        rect,
        text,
        clip,
        color,
        DEFAULT_FONT_SIZE,
        DEFAULT_LINE_HEIGHT,
    );
}

pub(super) fn draw_text_with_size(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
) {
    draw_text_with_size_and_style(
        frame,
        rect,
        text,
        clip,
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

pub(in crate::ui::retained_host::host_contract) fn draw_text_with_size_and_style(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
) {
    draw_text_with_size_and_style_impl(
        frame,
        rect,
        text,
        clip,
        color,
        font_size,
        line_height,
        style,
    );
}
