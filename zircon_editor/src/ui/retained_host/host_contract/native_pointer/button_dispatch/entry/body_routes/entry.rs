use crate::ui::retained_host::host_contract::data::{FrameRect, HostPresentationGeneration};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::NativePointerButtonState;
use super::input::BodyButtonRouteInput;
use super::sequence::dispatch_body_button_route_sequence;

pub(in super::super) fn dispatch_body_button_routes(
    ui: &UiHostWindow,
    presentation: &HostPresentationGeneration,
    state: NativePointerButtonState,
    button: UiPointerButton,
    button_id: i32,
    modifiers: UiInputModifiers,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    dispatch_body_button_route_sequence(BodyButtonRouteInput {
        ui,
        presentation,
        state,
        button,
        button_id,
        modifiers,
        x,
        y,
        cleared_text_input_frame,
    })
}
