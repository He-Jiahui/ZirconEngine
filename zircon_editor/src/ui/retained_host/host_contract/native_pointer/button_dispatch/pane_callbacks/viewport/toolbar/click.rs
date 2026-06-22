use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::surface_hit_test::ViewportToolbarPointerHit;

use super::super::super::super::super::routing::PanePointerRoute;

pub(super) fn invoke_viewport_toolbar_click(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    hit: &ViewportToolbarPointerHit,
) -> String {
    let control_id = hit.control_id.clone();
    pane_host.invoke_viewport_toolbar_pointer_clicked(
        hit.surface_key.clone(),
        hit.control_id.clone(),
        hit.control_x,
        hit.control_y,
        hit.control_width,
        hit.control_height,
        pointer.local_x,
        pointer.local_y,
    );
    control_id
}
