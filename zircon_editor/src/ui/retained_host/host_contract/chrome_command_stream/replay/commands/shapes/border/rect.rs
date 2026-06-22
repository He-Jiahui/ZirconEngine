use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_primitives::draw_rect_clipped;

pub(super) fn paint_rect_border_command(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    width: f32,
) {
    let width = width.ceil().max(1.0);
    for offset in 0..(width as u32) {
        let offset = offset as f32;
        paint_rect_border_segment(frame, rect, clip, color, offset, BorderSegment::Top);
        paint_rect_border_segment(frame, rect, clip, color, offset, BorderSegment::Bottom);
        paint_rect_border_segment(frame, rect, clip, color, offset, BorderSegment::Left);
        paint_rect_border_segment(frame, rect, clip, color, offset, BorderSegment::Right);
    }
}

fn paint_rect_border_segment(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    offset: f32,
    segment: BorderSegment,
) {
    draw_rect_clipped(
        frame,
        border_segment_rect(rect, offset, segment),
        clip,
        color,
    );
}

fn border_segment_rect(rect: &FrameRect, offset: f32, segment: BorderSegment) -> FrameRect {
    match segment {
        BorderSegment::Top => FrameRect {
            x: rect.x + offset,
            y: rect.y + offset,
            width: (rect.width - offset * 2.0).max(0.0),
            height: 1.0,
        },
        BorderSegment::Bottom => FrameRect {
            x: rect.x + offset,
            y: rect.y + rect.height - 1.0 - offset,
            width: (rect.width - offset * 2.0).max(0.0),
            height: 1.0,
        },
        BorderSegment::Left => FrameRect {
            x: rect.x + offset,
            y: rect.y + offset,
            width: 1.0,
            height: (rect.height - offset * 2.0).max(0.0),
        },
        BorderSegment::Right => FrameRect {
            x: rect.x + rect.width - 1.0 - offset,
            y: rect.y + offset,
            width: 1.0,
            height: (rect.height - offset * 2.0).max(0.0),
        },
    }
}

#[derive(Clone, Copy)]
enum BorderSegment {
    Top,
    Bottom,
    Left,
    Right,
}
