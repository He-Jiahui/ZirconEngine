use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::super::routing::ChromePointerRoute;
use super::super::super::tabs::dispatch_document_tab_press;

pub(super) fn dispatch_document_tab_route(ui: &UiHostWindow, route: &ChromePointerRoute) -> bool {
    let ChromePointerRoute::DocumentTab {
        surface_key,
        index,
        tab_x,
        tab_width,
        local_x,
        local_y,
        close,
    } = route
    else {
        return false;
    };

    dispatch_document_tab_press(
        ui,
        surface_key.clone(),
        *index,
        *tab_x,
        *tab_width,
        *local_x,
        *local_y,
        *close,
    );
    true
}
