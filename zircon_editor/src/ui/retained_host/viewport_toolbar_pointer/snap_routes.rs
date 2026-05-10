use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn snap_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    let route = match control_id {
        "snap.translate" | "translate_snap.cycle" => {
            ViewportToolbarPointerRoute::CycleTranslateSnap {
                surface_key: surface_key.to_string(),
            }
        }
        "snap.rotate" | "rotate_snap.cycle" => {
            ViewportToolbarPointerRoute::CycleRotateSnapDegrees {
                surface_key: surface_key.to_string(),
            }
        }
        "snap.scale" | "scale_snap.cycle" => ViewportToolbarPointerRoute::CycleScaleSnap {
            surface_key: surface_key.to_string(),
        },
        _ => return None,
    };
    Some(route)
}
