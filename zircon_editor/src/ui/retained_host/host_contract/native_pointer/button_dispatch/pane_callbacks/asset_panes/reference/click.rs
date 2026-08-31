use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::routing::{
    PaneAssetReferenceList, PaneAssetSurface, PanePointerRoute,
};
use super::super::super::super::super::NativePointerButtonState;

pub(super) fn dispatch_asset_reference_primary_click(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    mode: PaneAssetSurface,
    list_kind: PaneAssetReferenceList,
    state: NativePointerButtonState,
    button: UiPointerButton,
) {
    if state != NativePointerButtonState::Pressed || button != UiPointerButton::Primary {
        return;
    }
    pane_host.invoke_asset_reference_pointer_clicked(
        mode.as_str().into(),
        list_kind.as_str().into(),
        pointer.local_x,
        pointer.local_y,
        pointer.width,
        pointer.height,
    );
}
