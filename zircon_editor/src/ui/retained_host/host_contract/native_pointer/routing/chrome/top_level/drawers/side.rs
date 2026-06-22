use crate::ui::retained_host::host_contract::data::HostSideDockSurfaceData;

use super::super::super::super::ChromePointerRoute;
use super::super::super::tabs::route_drawer_header;

pub(super) fn route_side_drawer_header(
    surface_key: &str,
    dock: &HostSideDockSurfaceData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    route_drawer_header(
        surface_key,
        &dock.region_frame,
        &dock.header_frame,
        &dock.tab_frames,
        x,
        y,
    )
}
