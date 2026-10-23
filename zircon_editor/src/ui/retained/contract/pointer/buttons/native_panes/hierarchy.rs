use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::NativePointerButtonState;

pub(in crate::ui::retained_host::host_contract) fn dispatch_hierarchy_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
    host_kind: i32,
    button_id: i32,
) {
    pane_host.invoke_hierarchy_pointer_event(
        host_kind,
        button_id,
        pointer.local_x,
        pointer.local_y,
        pointer.width,
        pointer.height,
    );
    if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
        if std::env::var_os("ZIRCON_TRACE_EDITOR_INTERACTION").is_some() {
            eprintln!(
                "editor_interaction kind=hierarchy_press x={} y={} width={} height={}",
                pointer.local_x, pointer.local_y, pointer.width, pointer.height
            );
        }
        pane_host.invoke_hierarchy_pointer_clicked(
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        );
    }
}
