use super::data::{FrameRect, HostWindowPresentationData};
use super::paint_frame::{HostRecordedPaintCommand, HostRgbaFrame};
use super::paint_theme::PALETTE;
use super::paint_workbench::draw_workbench_presentation_commands;

const SHELL_BACKGROUND: [u8; 4] = PALETTE.shell_background;

pub(in crate::ui::retained_host::host_contract) fn record_host_frame_commands(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
    damage: Option<&FrameRect>,
) -> (Vec<HostRecordedPaintCommand>, Option<FrameRect>) {
    if width == 0 || height == 0 {
        return (Vec::new(), None);
    }

    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
    };
    let damage = damage.and_then(|damage| intersect_frames(damage, &frame_bounds));
    let mut frame = HostRgbaFrame::recording_only(width, height);
    if let Some(damage) = damage.as_ref() {
        frame.replace_paint_clip(Some(damage.clone()));
        frame.fill_rect(damage, SHELL_BACKGROUND);
    } else {
        frame.fill_rect(&frame_bounds, SHELL_BACKGROUND);
    }
    draw_workbench_presentation_commands(&mut frame, presentation);
    (frame.into_recorded_commands(), damage)
}

fn intersect_frames(left: &FrameRect, right: &FrameRect) -> Option<FrameRect> {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    let frame = FrameRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    };
    visible_frame(&frame).then_some(frame)
}

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}
