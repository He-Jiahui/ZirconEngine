use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn toggle_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    let route = match control_id {
        "toggle.lighting" | "preview_lighting.toggle" => {
            ViewportToolbarPointerRoute::TogglePreviewLighting {
                surface_key: surface_key.to_string(),
            }
        }
        "toggle.skybox" | "preview_skybox.toggle" => {
            ViewportToolbarPointerRoute::TogglePreviewSkybox {
                surface_key: surface_key.to_string(),
            }
        }
        "toggle.gizmos" | "gizmos.toggle" => ViewportToolbarPointerRoute::ToggleGizmosEnabled {
            surface_key: surface_key.to_string(),
        },
        _ => return None,
    };
    Some(route)
}
