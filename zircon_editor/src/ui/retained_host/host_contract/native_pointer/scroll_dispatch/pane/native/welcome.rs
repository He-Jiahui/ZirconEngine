use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;

use super::super::super::super::routing::PanePointerRoute;

pub(super) fn dispatch_welcome_scroll(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    delta: f32,
) -> bool {
    pane_host.invoke_welcome_recent_pointer_scrolled(
        pointer.local_x,
        pointer.local_y,
        delta,
        pointer.width,
        pointer.height,
    );
    true
}
