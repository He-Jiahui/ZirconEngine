use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn frame_selection_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    match control_id {
        "frame.selection" | "frame_selection" => {
            Some(ViewportToolbarPointerRoute::FrameSelection {
                surface_key: surface_key.to_string(),
            })
        }
        _ => None,
    }
}
