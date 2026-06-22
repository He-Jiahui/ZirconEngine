use super::super::super::super::data::HostResizeStateData;
use super::super::super::super::globals::UiHostContext;
use super::super::super::super::window::UiHostWindow;
use super::super::super::HOST_POINTER_DOWN;

pub(in crate::ui::retained_host::host_contract) fn arm_native_resize(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) {
    let host = ui.global::<UiHostContext>();
    host.set_resize_state(HostResizeStateData {
        resize_active: true,
        resize_pointer_x: x,
        resize_pointer_y: y,
        ..HostResizeStateData::default()
    });
    host.invoke_host_resize_pointer_event(HOST_POINTER_DOWN, x, y);
}
