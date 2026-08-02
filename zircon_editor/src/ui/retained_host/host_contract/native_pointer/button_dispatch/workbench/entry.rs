mod pressed;
mod released;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use self::pressed::dispatch_pressed_workbench_button;
use self::released::dispatch_released_workbench_button;
use super::super::super::NativePointerButtonState;
use super::super::super::routing::route_pointer_to_workbench_window;

pub(in super::super) fn dispatch_workbench_button(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    state: NativePointerButtonState,
    button: UiPointerButton,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    let hit = route_pointer_to_workbench_window(presentation, x, y)?;
    if state == NativePointerButtonState::Pressed {
        return dispatch_pressed_workbench_button(ui, hit, button, x, y, cleared_text_input_frame);
    }
    if state == NativePointerButtonState::Released {
        return Some(dispatch_released_workbench_button(hit));
    }
    None
}
