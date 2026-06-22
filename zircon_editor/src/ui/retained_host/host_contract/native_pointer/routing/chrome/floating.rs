mod header;

use crate::ui::retained_host::host_contract::data::HostFloatingWindowLayerData;

use self::header::route_floating_window_header_hit;
use super::super::ChromePointerRoute;

pub(super) fn route_floating_window_header(
    layer: &HostFloatingWindowLayerData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    for row in 0..layer.floating_windows.row_count() {
        let Some(window) = layer.floating_windows.row_data(row) else {
            continue;
        };
        if let Some(route) = route_floating_window_header_hit(&window, x, y) {
            return Some(route);
        }
    }

    None
}
