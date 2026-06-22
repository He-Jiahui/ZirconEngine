use crate::ui::retained_host::host_contract::data::{
    HostDragStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::routing::ChromePointerRoute;
use super::super::payload::tab_drag_payload_for_route;

pub(in crate::ui::retained_host::host_contract) fn arm_native_tab_drag(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
    x: f32,
    y: f32,
) {
    let Some((tab, source_group)) = tab_drag_payload_for_route(presentation, route) else {
        return;
    };
    ui.global::<UiHostContext>()
        .set_drag_state(HostDragStateData {
            drag_tab_id: tab.id,
            drag_tab_title: tab.title,
            drag_tab_icon_key: tab.icon_key,
            drag_source_group: source_group,
            drag_pointer_x: x,
            drag_pointer_y: y,
            ..HostDragStateData::default()
        });
}
