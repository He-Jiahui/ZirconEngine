use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::super::super::super::NativePointerButtonState;
use super::super::super::asset_panes::dispatch_asset_tree_button;

pub(super) fn dispatch_asset_tree_target(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
) -> bool {
    let PanePointerTarget::AssetTree(mode) = &pointer.target else {
        return false;
    };
    dispatch_asset_tree_button(pane_host, pointer, mode.clone(), state, button);
    true
}
