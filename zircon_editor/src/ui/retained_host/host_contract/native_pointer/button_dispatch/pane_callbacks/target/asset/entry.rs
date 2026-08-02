use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::NativePointerButtonState;
use super::super::super::super::super::routing::PanePointerRoute;
use super::content::dispatch_asset_content_target;
use super::reference::dispatch_asset_reference_target;
use super::tree::dispatch_asset_tree_target;

pub(in super::super) fn dispatch_asset_pane_target_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
    host_kind: i32,
    button_id: i32,
) -> bool {
    dispatch_asset_tree_target(pane_host, pointer, state, button)
        || dispatch_asset_content_target(pane_host, pointer, state, button, host_kind, button_id)
        || dispatch_asset_reference_target(pane_host, pointer, state, button, host_kind, button_id)
}
