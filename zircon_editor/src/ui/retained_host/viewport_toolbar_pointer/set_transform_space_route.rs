use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn set_transform_space_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    let space = match control_id {
        "space.local" | "transform.local" => "Local",
        "space.global" | "transform.global" => "Global",
        _ => return None,
    };
    Some(ViewportToolbarPointerRoute::SetTransformSpace {
        surface_key: surface_key.to_string(),
        space: space.to_string(),
    })
}
