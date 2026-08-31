use super::super::super::super::super::routing::PanePointerRoute;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;

pub(super) fn invoke_viewport_toolbar_click<'a>(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    surface_key: &'a str,
    control_id: Option<&'a str>,
) -> &'a str {
    pane_host.invoke_viewport_toolbar_pointer_clicked(
        surface_key.into(),
        pointer.local_x,
        pointer.local_y,
        pointer.width,
        pointer.height,
    );
    control_id.unwrap_or(surface_key)
}
