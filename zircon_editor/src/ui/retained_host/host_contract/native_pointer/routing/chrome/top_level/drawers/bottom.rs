use crate::ui::retained_host::host_contract::data::HostBottomDockSurfaceData;

use super::super::super::super::ChromePointerRoute;
use super::super::super::tabs::{route_dock_overflow, route_drawer_header};

pub(super) fn route_bottom_drawer_header(
    dock: &HostBottomDockSurfaceData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    if let Some(route) = route_dock_overflow(
        dock.surface_key.as_str(),
        &dock.region_frame,
        &dock.overflow_frame,
        x,
        y,
    ) {
        return Some(route);
    }
    route_drawer_header(
        dock.surface_key.as_str(),
        &dock.region_frame,
        &dock.header_frame,
        &dock.tab_frames,
        x,
        y,
    )
}
