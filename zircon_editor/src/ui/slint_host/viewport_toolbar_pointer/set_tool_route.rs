use super::viewport_toolbar_pointer_route::ViewportToolbarPointerRoute;

pub(super) fn set_tool_route(
    surface_key: &str,
    control_id: &str,
) -> Option<ViewportToolbarPointerRoute> {
    let tool = match control_id {
        "tool.drag" => "Drag",
        "tool.move" => "Move",
        "tool.rotate" => "Rotate",
        "tool.scale" => "Scale",
        _ => return None,
    };
    Some(ViewportToolbarPointerRoute::SetTool {
        surface_key: surface_key.to_string(),
        tool: tool.to_string(),
    })
}
