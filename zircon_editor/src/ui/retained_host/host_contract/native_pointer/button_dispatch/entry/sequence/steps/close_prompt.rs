use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::NativePointerButtonState;
use super::super::super::super::close_prompt::dispatch_close_prompt_button;
use super::super::super::input::ButtonDispatchInput;

pub(super) fn dispatch_close_prompt_step(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    input: &ButtonDispatchInput,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    dispatch_close_prompt_button(ui, &input.presentation, state, input.button, x, y)
}
