use super::super::super::super::data::HostResizeStateData;
use super::super::super::super::globals::UiHostContext;
use super::super::super::super::window::UiHostWindow;
use super::super::super::HOST_POINTER_DOWN;
use zircon_runtime_interface::ui::dispatch::UiPointerId;

pub(in crate::ui::retained_host::host_contract) fn arm_native_resize(
    ui: &UiHostWindow,
    pointer_id: UiPointerId,
    x: f32,
    y: f32,
) {
    let host = ui.global::<UiHostContext>();
    if host.native_primary_capture_active() {
        return;
    }
    host.set_resize_state(HostResizeStateData {
        capture_pointer_id: Some(pointer_id),
        resize_active: true,
        resize_pointer_x: x,
        resize_pointer_y: y,
        ..HostResizeStateData::default()
    });
    host.invoke_host_resize_pointer_event(HOST_POINTER_DOWN, x, y);
}
