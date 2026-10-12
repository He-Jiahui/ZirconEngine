use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::super::super::NativePointerButtonState;
use super::super::native_panes::{dispatch_hierarchy_button, dispatch_welcome_button};

pub(super) fn dispatch_native_pane_target_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
    host_kind: i32,
    button_id: i32,
) -> bool {
    match &pointer.target {
        PanePointerTarget::Hierarchy => {
            dispatch_hierarchy_button(pane_host, pointer, state, button, host_kind, button_id);
            true
        }
        PanePointerTarget::Welcome => {
            dispatch_welcome_button(pane_host, pointer, state, button);
            true
        }
        _ => false,
    }
}
