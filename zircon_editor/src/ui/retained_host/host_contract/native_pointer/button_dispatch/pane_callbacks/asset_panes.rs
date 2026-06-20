use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::routing::PanePointerRoute;
use super::super::super::NativePointerButtonState;

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

pub(in crate::ui::retained_host::host_contract) fn dispatch_asset_content_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    mode: SharedString,
    state: NativePointerButtonState,
    button: UiPointerButton,
    host_kind: i32,
    button_id: i32,
) {
    pane_host.invoke_asset_content_pointer_event(
        mode.clone(),
        host_kind,
        button_id,
        pointer.local_x,
        pointer.local_y,
        pointer.width,
        pointer.height,
    );
    if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
        pane_host.invoke_asset_content_pointer_clicked(
            mode,
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        );
    }
}

pub(in crate::ui::retained_host::host_contract) fn dispatch_asset_reference_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    mode: SharedString,
    list_kind: SharedString,
    state: NativePointerButtonState,
    button: UiPointerButton,
    host_kind: i32,
    button_id: i32,
) {
    pane_host.invoke_asset_reference_pointer_event(
        mode.clone(),
        list_kind.clone(),
        host_kind,
        button_id,
        pointer.local_x,
        pointer.local_y,
        pointer.width,
        pointer.height,
    );
    if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
        pane_host.invoke_asset_reference_pointer_clicked(
            mode,
            list_kind,
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        );
    }
}
