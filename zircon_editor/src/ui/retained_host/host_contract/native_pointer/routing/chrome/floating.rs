mod header;

use crate::ui::retained_host::host_contract::data::HostFloatingWindowLayerData;

use self::header::route_floating_window_header_hit;
use super::super::ChromePointerRoute;

pub(super) fn route_floating_window_header(
    layer: &HostFloatingWindowLayerData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    for window in layer.floating_windows.iter().rev() {
        if let Some(route) = route_floating_window_header_hit(window, x, y) {
            return Some(route);
        }
    }

    None
}

#[cfg(test)]
#[path = "tests/floating.rs"]
mod tests;
