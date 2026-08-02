use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::NativePointerButtonState;
use super::super::super::super::super::routing::PanePointerRoute;
use super::super::asset::dispatch_asset_pane_target_button;
use super::super::native::dispatch_native_pane_target_button;

pub(super) fn dispatch_callback_pane_targets(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
    host_kind: i32,
    button_id: i32,
) -> bool {
    dispatch_native_pane_target_button(pane_host, pointer, state, button, host_kind, button_id)
        || dispatch_asset_pane_target_button(
            pane_host, pointer, state, button, host_kind, button_id,
        )
}
