use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::intersect;
use super::super::paint_theme::PALETTE;
use super::commands::draw_workbench_presentation_commands;

const SHELL_BACKGROUND: [u8; 4] = PALETTE.shell_background;

pub(in crate::ui::retained_host::host_contract) fn paint_host_frame(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> HostRgbaFrame {
    if width == 0 || height == 0 {
        return HostRgbaFrame::empty(width, height);
    }

    let mut frame = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_full_frame_clear");
        HostRgbaFrame::filled(width, height, SHELL_BACKGROUND)
    };
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_draw_workbench_commands");
        draw_workbench_presentation_commands(&mut frame, presentation);
    }
    frame
}

pub(in crate::ui::retained_host::host_contract) fn repaint_host_frame_region(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    damage: &FrameRect,
) -> Option<FrameRect> {
    if frame.width() == 0 || frame.height() == 0 {
        return None;
    }
    let frame_bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: frame.width() as f32,
        height: frame.height() as f32,
    };
    let damage = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_region_clip");
        intersect(damage, &frame_bounds)?
    };

    // The retained backbuffer is the authoritative previous frame, and the
    // active paint clip makes every painter operation respect damage.
    let previous_clip = frame.replace_paint_clip(Some(damage.clone()));
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_region_clear");
        frame.fill_rect(&damage, SHELL_BACKGROUND);
    }
    {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "painter_region_draw_workbench_commands"
        );
        draw_workbench_presentation_commands(frame, presentation);
    }
    frame.replace_paint_clip(previous_clip);
    Some(damage)
}
