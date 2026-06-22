mod bottom;
mod side;

use crate::ui::retained_host::host_contract::data::HostWindowSceneData;

use self::bottom::route_bottom_drawer_header;
use self::side::route_side_drawer_header;
use super::super::super::ChromePointerRoute;

pub(super) fn route_drawer_headers(
    scene: &HostWindowSceneData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    if let Some(route) = route_side_drawer_header("left", &scene.left_dock, x, y) {
        return Some(route);
    }
    if let Some(route) = route_side_drawer_header("right", &scene.right_dock, x, y) {
        return Some(route);
    }
    route_bottom_drawer_header(&scene.bottom_dock, x, y)
}
