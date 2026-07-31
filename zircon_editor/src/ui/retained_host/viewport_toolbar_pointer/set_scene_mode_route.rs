use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn set_scene_mode_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    let mode = match control_id {
        "mode.select" => "Select".to_string(),
        "mode.move" => "Transform.Move".to_string(),
        "mode.rotate" => "Transform.Rotate".to_string(),
        "mode.scale" => "Transform.Scale".to_string(),
        _ => control_id
            .strip_prefix("mode.custom:")
            .filter(|mode_id| !mode_id.is_empty())
            .map(|mode_id| format!("Custom:{mode_id}"))?,
    };
    Some(ViewportToolbarPointerRoute::ActivateSceneMode {
        surface_key: surface_key.to_string(),
        mode,
    })
}
