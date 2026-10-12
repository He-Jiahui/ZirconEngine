use crate::ui::retained_host::host_contract::data::{FrameRect, HostSideDockSurfaceData};
use crate::ui::retained_host::host_contract::profiling_artifacts::UiProfileNamedFrame;

use super::super::frame_math::{is_visible_frame, push_named_frame, translated};

pub(in crate::ui::retained_host::host_contract) fn collect_activity_rail_buttons(
    surface: &str,
    dock: &HostSideDockSurfaceData,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    if dock.rail_width_px <= 0.0 || !is_visible_frame(&dock.region_frame) {
        return;
    }
    let rail_x = if dock.rail_before_panel {
        dock.region_frame.x
    } else {
        dock.region_frame.x + (dock.region_frame.width - dock.rail_width_px).max(0.0)
    };
    let rail = FrameRect {
        x: rail_x,
        y: dock.region_frame.y,
        width: dock.rail_width_px.min(dock.region_frame.width.max(0.0)),
        height: dock.region_frame.height,
    };
    for row in 0..dock.rail_button_frames.row_count() {
        let Some(button) = dock.rail_button_frames.row_data(row) else {
            continue;
        };
        let frame = translated(&button.frame, rail.x, rail.y);
        push_named_frame(
            out,
            format!("activity_rail.{surface}.{}", button.control_id),
            "activity_rail_button",
            surface,
            frame,
            None,
        );
    }
}
