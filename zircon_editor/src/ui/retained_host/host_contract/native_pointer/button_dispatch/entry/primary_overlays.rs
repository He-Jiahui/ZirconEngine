use crate::ui::retained_host::host_contract::data::{FrameRect, HostPresentationGeneration};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::NativePointerButtonState;
use super::super::primary_press::dispatch_primary_press_overlays;

pub(super) fn dispatch_primary_press_overlays_if_pressed(
    ui: &UiHostWindow,
    presentation: &HostPresentationGeneration,
    state: NativePointerButtonState,
    button: UiPointerButton,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    if state != NativePointerButtonState::Pressed || button != UiPointerButton::Primary {
        return None;
    }
    dispatch_primary_press_overlays(ui, presentation, x, y, cleared_text_input_frame)
}
