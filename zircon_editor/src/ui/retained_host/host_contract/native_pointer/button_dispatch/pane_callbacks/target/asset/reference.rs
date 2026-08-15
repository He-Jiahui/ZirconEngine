use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::super::super::super::NativePointerButtonState;
use super::super::super::asset_panes::dispatch_asset_reference_button;

pub(super) fn dispatch_asset_reference_target(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
    host_kind: i32,
    button_id: i32,
) -> bool {
    let PanePointerTarget::AssetReference(mode, list_kind) = &pointer.target else {
        return false;
    };
    dispatch_asset_reference_button(
        pane_host,
        pointer,
        mode.clone(),
        list_kind.clone(),
        state,
        button,
        host_kind,
        button_id,
    );
    true
}
