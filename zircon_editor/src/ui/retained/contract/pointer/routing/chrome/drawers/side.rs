use crate::ui::retained_host::host_contract::data::{FrameRect, HostSideDockSurfaceData};

use super::super::super::super::ChromePointerRoute;
use super::super::super::tabs::{route_dock_overflow, route_drawer_header};

pub(super) fn route_side_drawer_header(
    surface_key: &str,
    dock: &HostSideDockSurfaceData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let panel_origin = FrameRect {
        x: if dock.rail_before_panel {
            dock.region_frame.x + dock.rail_width_px
        } else {
            dock.region_frame.x
        },
        y: dock.region_frame.y,
        width: dock.panel_width_px,
        height: dock.region_frame.height,
    };
    if let Some(route) = route_dock_overflow(surface_key, &panel_origin, &dock.overflow_frame, x, y)
    {
        return Some(route);
    }
    route_drawer_header(
        surface_key,
        &panel_origin,
        &dock.header_frame,
        &dock.tab_frames,
        x,
        y,
    )
}
