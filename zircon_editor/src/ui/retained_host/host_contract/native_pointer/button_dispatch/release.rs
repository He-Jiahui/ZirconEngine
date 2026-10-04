use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiPointerId;

use super::super::drag_resize::{finish_native_resize, finish_native_tab_drag};

pub(super) fn finish_primary_capture(
    ui: &UiHostWindow,
    pointer_id: UiPointerId,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    if let Some(result) = finish_native_resize(ui, pointer_id, x, y) {
        return Some(result);
    }
    finish_native_tab_drag(ui, pointer_id, x, y)
}
