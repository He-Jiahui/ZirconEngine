use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::NativePointerButtonState;
use super::super::super::input::ButtonDispatchInput;
use super::super::super::release_capture::finish_primary_capture_if_released;

pub(super) fn finish_release_capture_step(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    input: &ButtonDispatchInput,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    finish_primary_capture_if_released(ui, state, input.button, x, y)
}
