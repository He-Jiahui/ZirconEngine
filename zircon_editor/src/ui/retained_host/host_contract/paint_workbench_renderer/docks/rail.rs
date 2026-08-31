use super::super::super::data::{FrameRect, HostSideDockSurfaceData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::translated;
use super::super::super::paint_primitives::draw_border_clipped;
use super::palette::current_dock_chrome_palette;

pub(in crate::ui::retained_host::host_contract) fn draw_active_rail_marker(
    frame: &mut HostRgbaFrame,
    dock: &HostSideDockSurfaceData,
    rail_origin: &FrameRect,
) {
    if dock.rail_active_control_id.is_empty() {
        return;
    }
    let palette = current_dock_chrome_palette();
    for row in 0..dock.rail_button_frames.row_count() {
        let Some(control) = dock.rail_button_frames.get(row) else {
            continue;
        };
        if control.control_id.as_str() == dock.rail_active_control_id.as_str()
            || dock
                .rail_active_control_id
                .as_str()
                .ends_with(control.control_id.as_str())
        {
            let marker = translated(&control.frame, rail_origin.x, rail_origin.y);
            draw_border_clipped(frame, marker, Some(rail_origin), palette.accent);
        }
    }
}
