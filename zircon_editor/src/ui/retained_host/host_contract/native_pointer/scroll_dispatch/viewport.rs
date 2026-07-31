use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::{VIEWPORT_POINTER_BUTTON_NONE, VIEWPORT_POINTER_SCROLL};

pub(super) fn dispatch_viewport_pointer_scroll(
    pane_host: &PaneSurfaceHostContext,
    local_x: f32,
    local_y: f32,
    delta: f32,
) -> NativePointerDispatchResult {
    pane_host.invoke_viewport_pointer_event(
        VIEWPORT_POINTER_SCROLL,
        VIEWPORT_POINTER_BUTTON_NONE,
        local_x,
        local_y,
        delta,
        false,
        false,
    );
    NativePointerDispatchResult::idle()
}
