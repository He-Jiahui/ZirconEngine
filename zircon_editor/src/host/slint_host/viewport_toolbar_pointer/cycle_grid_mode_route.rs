use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn cycle_grid_mode_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    match control_id {
        "grid.cycle" => Some(ViewportToolbarPointerRoute::CycleGridMode {
            surface_key: surface_key.to_string(),
        }),
        _ => None,
    }
}
