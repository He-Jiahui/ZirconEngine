use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::drag_resize::{
    dispatch_native_resize_move, dispatch_native_tab_drag_move,
};

pub(super) fn dispatch_pointer_move_capture(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    if let Some(result) = dispatch_native_resize_move(ui, x, y) {
        return Some(result);
    }
    dispatch_native_tab_drag_move(ui, x, y)
}
