use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::NativePointerButtonState;
use super::super::super::body_routes::dispatch_body_button_routes;
use super::super::super::input::ButtonDispatchInput;

pub(super) fn dispatch_body_route_step(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    input: ButtonDispatchInput,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    dispatch_body_button_routes(
        ui,
        &input.presentation,
        state,
        input.button,
        input.button_id,
        input.modifiers,
        x,
        y,
        cleared_text_input_frame,
    )
}
