use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;

use super::super::super::routing::{PanePointerRoute, PanePointerTarget};

pub(super) fn dispatch_native_pane_move(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
) {
    match &pointer.target {
        PanePointerTarget::Hierarchy => pane_host.invoke_hierarchy_pointer_moved(
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        ),
        PanePointerTarget::Welcome => pane_host.invoke_welcome_recent_pointer_moved(
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        ),
        _ => {}
    }
}
