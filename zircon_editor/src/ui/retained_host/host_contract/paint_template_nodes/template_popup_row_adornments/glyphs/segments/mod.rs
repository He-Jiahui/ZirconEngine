use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::local_rect;

mod style;

use style::popup_adornment_segment_style;

#[derive(Clone, Copy)]
pub(super) struct PopupAdornmentSegmentSpec {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

pub(super) const fn popup_adornment_segment(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> PopupAdornmentSegmentSpec {
    PopupAdornmentSegmentSpec {
        x,
        y,
        width,
        height,
    }
}

impl PopupAdornmentSegmentSpec {
    fn frame(self, origin: &FrameRect) -> FrameRect {
        local_rect(
            origin,
            f32::from(self.x),
            f32::from(self.y),
            f32::from(self.width),
            f32::from(self.height),
        )
    }
}

pub(super) fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[PopupAdornmentSegmentSpec],
) {
    let style = popup_adornment_segment_style(color);
    for segment in segments {
        commands.push(HostPaintCommand::quad(
            segment.frame(rect),
            Some(clip.clone()),
            order,
            Some(style.fill),
            style.border,
            style.border_width,
            style.radius,
            opacity,
        ));
    }
}
