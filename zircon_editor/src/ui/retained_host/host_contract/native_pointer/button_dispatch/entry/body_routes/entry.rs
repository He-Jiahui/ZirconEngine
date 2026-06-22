use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::NativePointerButtonState;
use super::input::BodyButtonRouteInput;
use super::sequence::dispatch_body_button_route_sequence;

pub(in super::super) fn dispatch_body_button_routes(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    state: NativePointerButtonState,
    button: UiPointerButton,
    button_id: i32,
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
        x,
        y,
        cleared_text_input_frame,
    })
}
