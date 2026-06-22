use crate::ui::retained_host::host_contract::data::HostDragStateData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::HOST_POINTER_MOVE;

pub(super) fn dispatch_active_tab_drag_move(
    ui: &UiHostWindow,
    mut drag_state: HostDragStateData,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    drag_state.drag_pointer_x = x;
    drag_state.drag_pointer_y = y;
    let host = ui.global::<UiHostContext>();
    host.set_drag_state(drag_state);
    host.invoke_host_drag_pointer_event(HOST_POINTER_MOVE, x, y);
    NativePointerDispatchResult::idle()
}
