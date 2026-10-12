use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiPointerId;

use super::super::super::drag_resize::arm_native_resize;

pub(super) fn dispatch_resize_press(ui: &UiHostWindow, pointer_id: UiPointerId, x: f32, y: f32) {
    arm_native_resize(ui, pointer_id, x, y);
}
