use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::NativePointerButtonState;
use super::super::super::input::ButtonDispatchInput;
use super::super::super::primary_overlays::dispatch_primary_press_overlays_if_pressed;

pub(super) fn dispatch_primary_overlay_step(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    input: &ButtonDispatchInput,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    dispatch_primary_press_overlays_if_pressed(
        ui,
        &input.presentation,
        state,
        input.button,
        x,
        y,
        cleared_text_input_frame,
    )
}
