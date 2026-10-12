use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::NativePointerButtonState;
use super::super::super::super::asset_deletion_blocker::dispatch_asset_deletion_blocker_button;
use super::super::super::input::ButtonDispatchInput;

pub(super) fn dispatch_asset_deletion_blocker_step(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    input: &ButtonDispatchInput,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    dispatch_asset_deletion_blocker_button(
        ui,
        input.presentation.structure(),
        state,
        input.button,
        x,
        y,
    )
}
