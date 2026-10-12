use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::routing::ChromePointerRoute;

#[path = "../../../../../contract/pointer/buttons/entry/tabs/dock_overflow.rs"]
mod dock_overflow;
#[path = "tabs/document.rs"]
mod document;
#[path = "tabs/drawer.rs"]
mod drawer;
#[path = "tabs/host_page.rs"]
mod host_page;

pub(super) fn dispatch_chrome_tab_press(ui: &UiHostWindow, route: &ChromePointerRoute) -> bool {
    dock_overflow::dispatch_dock_overflow_route(ui, route)
        || host_page::dispatch_host_page_tab_route(ui, route)
        || document::dispatch_document_tab_route(ui, route)
        || drawer::dispatch_drawer_header_tab_route(ui, route)
}
