use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::input_policy::DRAG_START_DISTANCE_PX;

use super::super::super::super::super::HOST_POINTER_DOWN;

pub(super) fn start_tab_drag_move(
    ui: &UiHostWindow,
    pointer_x: f32,
    pointer_y: f32,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let distance_x = x - pointer_x;
    let distance_y = y - pointer_y;
    if distance_x.hypot(distance_y) < DRAG_START_DISTANCE_PX {
        return NativePointerDispatchResult::idle();
    }
    let host = ui.global::<UiHostContext>();
    host.activate_drag_at(x, y);
    host.invoke_host_drag_pointer_event(HOST_POINTER_DOWN, x, y);
    NativePointerDispatchResult::idle()
}
