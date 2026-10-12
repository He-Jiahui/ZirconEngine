use crate::ui::retained_host::host_contract::data::{
    HostDragStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::routing::ChromePointerRoute;
use super::super::payload::tab_drag_payload_for_route;
use zircon_runtime_interface::ui::dispatch::UiPointerId;

pub(in crate::ui::retained_host::host_contract) fn arm_native_tab_drag(
    ui: &UiHostWindow,
    pointer_id: UiPointerId,
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
    x: f32,
    y: f32,
) {
    let Some((tab, source_group)) = tab_drag_payload_for_route(presentation, route) else {
        return;
    };
    let host = ui.global::<UiHostContext>();
    if host.native_primary_capture_active() {
        return;
    }
    host.set_drag_state(HostDragStateData {
        capture_pointer_id: Some(pointer_id),
        drag_tab_id: tab.id.clone(),
        drag_tab_title: tab.title.clone(),
        drag_tab_icon_key: tab.icon_key.clone(),
        drag_source_group: source_group.clone(),
        drag_pointer_x: x,
        drag_pointer_y: y,
        ..HostDragStateData::default()
    });
}
