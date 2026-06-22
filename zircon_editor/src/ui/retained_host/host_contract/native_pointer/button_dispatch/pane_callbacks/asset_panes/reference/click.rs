use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::super::NativePointerButtonState;

pub(super) fn dispatch_asset_reference_primary_click(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    mode: SharedString,
    list_kind: SharedString,
    state: NativePointerButtonState,
    button: UiPointerButton,
) {
    if state != NativePointerButtonState::Pressed || button != UiPointerButton::Primary {
        return;
    }
    pane_host.invoke_asset_reference_pointer_clicked(
        mode,
        list_kind,
        pointer.local_x,
        pointer.local_y,
        pointer.width,
        pointer.height,
    );
}
