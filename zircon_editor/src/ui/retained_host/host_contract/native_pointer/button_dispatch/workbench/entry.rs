mod pressed;
mod released;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostPresentationGeneration};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use self::pressed::dispatch_pressed_workbench_button;
use self::released::dispatch_released_workbench_button;
use super::super::super::routing::route_pointer_to_workbench_generation;
use super::super::super::NativePointerButtonState;

pub(in super::super) fn dispatch_workbench_button(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    state: NativePointerButtonState,
    button: UiPointerButton,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    let hit = route_pointer_to_workbench_generation(generation, x, y)?;
    if state == NativePointerButtonState::Pressed {
        return dispatch_pressed_workbench_button(ui, hit, button, x, y, cleared_text_input_frame);
    }
    if state == NativePointerButtonState::Released {
        return Some(dispatch_released_workbench_button(hit));
    }
    None
}
