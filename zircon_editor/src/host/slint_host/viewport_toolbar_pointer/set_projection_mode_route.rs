use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn set_projection_mode_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    let mode = match control_id {
        "projection.perspective" => "Perspective",
        "projection.orthographic" => "Orthographic",
        _ => return None,
    };
    Some(ViewportToolbarPointerRoute::SetProjectionMode {
        surface_key: surface_key.to_string(),
        mode: mode.to_string(),
    })
}
