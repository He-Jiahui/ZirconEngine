use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::NativePointerButtonState;
use super::super::super::super::text_focus::clear_focused_text_input_on_primary_press;
use super::super::super::input::ButtonDispatchInput;

pub(super) fn clear_text_focus_step(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    input: &ButtonDispatchInput,
) -> Option<FrameRect> {
    clear_focused_text_input_on_primary_press(ui, state, input.button)
}
