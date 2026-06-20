use super::super::chrome::is_viewport_chrome_node;

const VIEWPORT_CONTROL_PREFIX: &str = "WorkbenchViewport";

pub(super) fn is_viewport_scene_candidate(id: &str) -> bool {
    id.starts_with(VIEWPORT_CONTROL_PREFIX)
        && !is_viewport_chrome_node(id)
        && !is_viewport_axis_label_or_gizmo(id)
}

fn is_viewport_axis_label_or_gizmo(id: &str) -> bool {
    matches!(
        id,
        "WorkbenchViewportAxisXLabel"
            | "WorkbenchViewportAxisYLabel"
            | "WorkbenchViewportGizmoX"
            | "WorkbenchViewportGizmoY"
            | "WorkbenchViewportGizmoZ"
    )
}
