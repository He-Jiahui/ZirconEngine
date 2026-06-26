use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::host_page_pointer::HOST_PAGE_OVERFLOW_POINTER_INDEX;

use super::super::super::super::super::routing::ChromePointerRoute;
use super::super::super::tabs::dispatch_host_page_tab_press;

pub(super) fn dispatch_host_page_tab_route(ui: &UiHostWindow, route: &ChromePointerRoute) -> bool {
    match route {
        ChromePointerRoute::HostPageTab {
            index,
            tab_x,
            tab_width,
            local_x,
            local_y,
        } => {
            dispatch_host_page_tab_press(ui, *index as i32, *tab_x, *tab_width, *local_x, *local_y);
            true
        }
        ChromePointerRoute::HostPageOverflow {
            tab_x,
            tab_width,
            local_x,
            local_y,
        } => {
            dispatch_host_page_tab_press(
                ui,
                HOST_PAGE_OVERFLOW_POINTER_INDEX,
                *tab_x,
                *tab_width,
                *local_x,
                *local_y,
            );
            true
        }
        _ => false,
    }
}
