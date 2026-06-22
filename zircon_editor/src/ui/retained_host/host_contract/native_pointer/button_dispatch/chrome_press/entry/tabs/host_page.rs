use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::routing::ChromePointerRoute;
use super::super::super::tabs::dispatch_host_page_tab_press;

pub(super) fn dispatch_host_page_tab_route(ui: &UiHostWindow, route: &ChromePointerRoute) -> bool {
    let ChromePointerRoute::HostPageTab {
        index,
        tab_x,
        tab_width,
        local_x,
        local_y,
    } = route
    else {
        return false;
    };

    dispatch_host_page_tab_press(ui, *index, *tab_x, *tab_width, *local_x, *local_y);
    true
}
