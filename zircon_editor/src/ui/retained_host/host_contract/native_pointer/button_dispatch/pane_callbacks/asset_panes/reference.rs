mod click;

use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::NativePointerButtonState;
use super::super::super::super::routing::PanePointerRoute;

use self::click::dispatch_asset_reference_primary_click;

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
    dispatch_asset_reference_primary_click(pane_host, pointer, mode, list_kind, state, button);
}
