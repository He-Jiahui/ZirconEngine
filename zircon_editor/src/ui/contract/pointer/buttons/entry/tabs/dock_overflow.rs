use crate::ui::retained_host::host_contract::data::{
    HostDockOverflowMenuStateData, HostMenuStateData, HostPageOverflowMenuStateData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::routing::ChromePointerRoute;

pub(super) fn dispatch_dock_overflow_route(ui: &UiHostWindow, route: &ChromePointerRoute) -> bool {
    let ChromePointerRoute::DockOverflow { surface_key } = route else {
        return false;
    };
    let generation = ui.get_host_presentation_generation();
    let current = generation.dock_overflow_menu_state();
    let close = current.open && current.surface_key == *surface_key;
    let host = ui.global::<UiHostContext>();
    host.set_menu_state(HostMenuStateData::default());
    host.set_host_page_overflow_menu_state(HostPageOverflowMenuStateData::default());
    host.set_host_dock_overflow_menu_state(if close {
        HostDockOverflowMenuStateData::default()
    } else {
        HostDockOverflowMenuStateData {
            open: true,
            surface_key: surface_key.clone(),
            hovered_tab_index: -1,
            scroll_offset: 0.0,
        }
    });
    true
}
