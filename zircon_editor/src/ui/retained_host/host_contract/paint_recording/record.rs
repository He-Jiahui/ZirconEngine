use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::paint_frame::{HostRecordedPaintCommand, HostRgbaFrame};
use super::super::paint_theme::PALETTE;
use super::super::paint_workbench::draw_workbench_presentation_commands;
use super::damage::{clip_damage_to_frame, frame_bounds};

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

    let frame_bounds = frame_bounds(width, height);
    let damage = clip_damage_to_frame(damage, &frame_bounds);
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
