use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::viewport::dispatch_viewport_pointer_scroll;

pub(super) fn dispatch_viewport_pane_scroll(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    delta: f32,
) -> Option<NativePointerDispatchResult> {
    let PanePointerTarget::Viewport(_) = &pointer.target else {
        return None;
    };
    Some(dispatch_viewport_pointer_scroll(
        pane_host,
        pointer.local_x,
        pointer.local_y,
        delta,
    ))
}
