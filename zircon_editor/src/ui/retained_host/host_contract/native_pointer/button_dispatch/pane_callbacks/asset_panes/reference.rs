mod click;

use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::routing::{
    PaneAssetReferenceList, PaneAssetSurface, PanePointerRoute,
};
use super::super::super::super::NativePointerButtonState;

use self::click::dispatch_asset_reference_primary_click;

pub(in crate::ui::retained_host::host_contract) fn dispatch_asset_reference_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    mode: PaneAssetSurface,
    list_kind: PaneAssetReferenceList,
    state: NativePointerButtonState,
    button: UiPointerButton,
    host_kind: i32,
    button_id: i32,
) {
    pane_host.invoke_asset_reference_pointer_event(
        mode.as_str().into(),
        list_kind.as_str().into(),
        host_kind,
        button_id,
        pointer.local_x,
        pointer.local_y,
        pointer.width,
        pointer.height,
    );
    dispatch_asset_reference_primary_click(pane_host, pointer, mode, list_kind, state, button);
}
