use crate::ui::retained_host::host_contract::data::HostDragStateData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::HOST_POINTER_DOWN;

const TAB_DRAG_START_DISTANCE_PX: f32 = 4.0;

pub(super) fn start_tab_drag_move(
    ui: &UiHostWindow,
    mut drag_state: HostDragStateData,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let distance_x = x - drag_state.drag_pointer_x;
    let distance_y = y - drag_state.drag_pointer_y;
    if distance_x.hypot(distance_y) < TAB_DRAG_START_DISTANCE_PX {
        return NativePointerDispatchResult::idle();
    }
    drag_state.drag_active = true;
    drag_state.drag_pointer_x = x;
    drag_state.drag_pointer_y = y;
    let host = ui.global::<UiHostContext>();
    host.set_drag_state(drag_state);
    host.invoke_host_drag_pointer_event(HOST_POINTER_DOWN, x, y);
    NativePointerDispatchResult::idle()
}
