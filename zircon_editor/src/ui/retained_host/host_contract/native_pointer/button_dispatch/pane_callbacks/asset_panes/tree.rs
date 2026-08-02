use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::NativePointerButtonState;
use super::super::super::super::routing::PanePointerRoute;

pub(in crate::ui::retained_host::host_contract) fn dispatch_asset_tree_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    mode: SharedString,
    state: NativePointerButtonState,
    button: UiPointerButton,
) {
    if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
        pane_host.invoke_asset_tree_pointer_clicked(
            mode,
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        );
    }
}
