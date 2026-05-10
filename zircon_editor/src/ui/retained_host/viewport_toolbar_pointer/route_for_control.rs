use super::align_view_route::align_view_route;
use super::cycle_display_mode_route::cycle_display_mode_route;
use super::cycle_grid_mode_route::cycle_grid_mode_route;
use super::frame_selection_route::frame_selection_route;
use super::play_mode_route::play_mode_route;
use super::set_projection_mode_route::set_projection_mode_route;
use super::set_tool_route::set_tool_route;
use super::set_transform_space_route::set_transform_space_route;
use super::snap_routes::snap_route;
use super::toggle_routes::toggle_route;
use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn route_for_control(
    surface_key: &str,
    control_id: &str,
) -> Result<ViewportToolbarPointerRoute, String> {
    for route in [
        set_tool_route(surface_key, control_id),
        set_transform_space_route(surface_key, control_id),
        set_projection_mode_route(surface_key, control_id),
        align_view_route(surface_key, control_id),
        cycle_display_mode_route(surface_key, control_id),
        cycle_grid_mode_route(surface_key, control_id),
        snap_route(surface_key, control_id),
        toggle_route(surface_key, control_id),
        frame_selection_route(surface_key, control_id),
        play_mode_route(surface_key, control_id),
    ] {
        if let Some(route) = route {
            return Ok(route);
        }
    }

    Err(format!(
        "Unknown viewport toolbar control {control_id} on surface {surface_key}"
    ))
}
