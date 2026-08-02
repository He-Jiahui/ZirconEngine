use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::NativePointerButtonState;
use super::super::super::super::routing::PanePointerRoute;

pub(in crate::ui::retained_host::host_contract) fn dispatch_welcome_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
) {
    if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
        pane_host.invoke_welcome_recent_pointer_clicked(
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        );
    }
}
