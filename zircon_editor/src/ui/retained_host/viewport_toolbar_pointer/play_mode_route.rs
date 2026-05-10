use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn play_mode_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    match control_id {
        "EnterPlayMode" => Some(ViewportToolbarPointerRoute::EnterPlayMode {
            surface_key: surface_key.to_string(),
        }),
        "ExitPlayMode" => Some(ViewportToolbarPointerRoute::ExitPlayMode {
            surface_key: surface_key.to_string(),
        }),
        _ => None,
    }
}
