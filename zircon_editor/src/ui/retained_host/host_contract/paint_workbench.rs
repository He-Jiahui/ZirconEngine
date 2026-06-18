use super::data::HostWindowPresentationData;
use super::paint_frame::HostRgbaFrame;
use super::paint_theme::PALETTE;
use super::paint_workbench_impl as workbench;

#[cfg(test)]
use super::data::FrameRect;
#[cfg(test)]
use super::paint_geometry::intersect;

const SHELL_BACKGROUND: [u8; 4] = PALETTE.shell_background;

pub(in crate::ui::retained_host::host_contract) fn draw_workbench_presentation_commands(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    if workbench::draws_componentized_workbench_window(presentation) {
        workbench::draw_componentized_workbench_window(frame, presentation);
    } else {
        workbench::draw_legacy_workbench_window(frame, presentation);
    }
}

#[cfg(test)]
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
    if workbench::draws_componentized_workbench_window(presentation) {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "painter_draw_componentized_workbench"
        );
        workbench::draw_componentized_workbench_window(&mut frame, presentation);
    } else {
        workbench::draw_legacy_workbench_window_profiled(
            &mut frame,
            width,
            height,
            presentation,
            "painter_resolve_root_frames",
            "painter_draw_root_skeleton",
            "painter_draw_host_scene",
        );
    }
    frame
}

#[cfg(test)]
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
    if workbench::draws_componentized_workbench_window(presentation) {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "painter_region_draw_componentized_workbench"
        );
        workbench::draw_componentized_workbench_window(frame, presentation);
    } else {
        workbench::draw_legacy_workbench_window_profiled(
            frame,
            frame.width(),
            frame.height(),
            presentation,
            "painter_region_resolve_root_frames",
            "painter_region_draw_root_skeleton",
            "painter_region_draw_host_scene",
        );
    }
    frame.replace_paint_clip(previous_clip);
    Some(damage)
}
