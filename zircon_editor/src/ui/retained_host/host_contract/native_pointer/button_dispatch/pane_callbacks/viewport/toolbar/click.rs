use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::primitives::SharedString;

use super::super::super::super::super::routing::PanePointerRoute;

pub(super) fn invoke_viewport_toolbar_click(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    surface_key: &SharedString,
    control_id: Option<&SharedString>,
) -> String {
    pane_host.invoke_viewport_toolbar_pointer_clicked(
        surface_key.clone(),
        pointer.local_x,
        pointer.local_y,
        pointer.width,
        pointer.height,
    );
    control_id.unwrap_or(surface_key).to_string()
}
