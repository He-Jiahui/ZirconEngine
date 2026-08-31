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
mod tests {
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::data::{
        FloatingWindowData, FrameRect, HostFloatingWindowLayerData,
    };

    use super::super::super::ChromePointerRoute;
    use super::route_floating_window_header;

    #[test]
    fn overlapping_headers_route_to_the_topmost_floating_window() {
        let window = |window_id: &str| FloatingWindowData {
            window_id: window_id.into(),
            frame: FrameRect {
                x: 10.0,
                y: 10.0,
                width: 200.0,
                height: 120.0,
            },
            header_frame: FrameRect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 28.0,
            },
            ..FloatingWindowData::default()
        };
        let layer = HostFloatingWindowLayerData {
            floating_windows: model_rc(vec![window("back"), window("front")]),
            ..HostFloatingWindowLayerData::default()
        };

        let route = route_floating_window_header(&layer, 20.0, 20.0)
            .expect("overlapping floating header route");

        assert!(matches!(
            route,
            ChromePointerRoute::FloatingWindowHeader { window_id }
                if window_id == "front"
        ));
    }
}
