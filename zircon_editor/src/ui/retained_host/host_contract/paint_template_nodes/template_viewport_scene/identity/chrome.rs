pub(super) fn is_viewport_chrome_node(id: &str) -> bool {
    matches!(
        id,
        "WorkbenchViewportPanel"
            | "WorkbenchViewportToolbar"
            | "WorkbenchViewportToolbarFill"
            | "WorkbenchViewportMode"
            | "WorkbenchViewportLit"
            | "WorkbenchViewportAngle"
            | "WorkbenchViewportSpeed"
    )
}
