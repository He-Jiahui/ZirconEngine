use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::NativePointerButtonState;
use super::super::release::finish_primary_capture;

pub(super) fn finish_primary_capture_if_released(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    button: UiPointerButton,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    if state != NativePointerButtonState::Released || button != UiPointerButton::Primary {
        return None;
    }
    finish_primary_capture(ui, x, y)
}
