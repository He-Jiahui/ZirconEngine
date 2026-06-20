use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::drag_resize::arm_native_resize;
use super::super::routing::ChromePointerRoute;

pub(in crate::ui::retained_host::host_contract) fn dispatch_chrome_press(
    ui: &UiHostWindow,
    route: ChromePointerRoute,
    x: f32,
    y: f32,
) {
    let host = ui.global::<UiHostContext>();
    match route {
        ChromePointerRoute::ActivityRail {
            side,
            local_x,
            local_y,
        } => {
            host.invoke_activity_rail_pointer_clicked(side, local_x, local_y);
        }
        ChromePointerRoute::HostPageTab {
            index,
            tab_x,
            tab_width,
            local_x,
            local_y,
        } => {
            host.invoke_host_page_pointer_clicked(index as i32, tab_x, tab_width, local_x, local_y)
        }
        ChromePointerRoute::DocumentTab {
            surface_key,
            index,
            tab_x,
            tab_width,
            local_x,
            local_y,
            close,
        } => {
            if close {
                host.invoke_document_tab_close_pointer_clicked(
                    surface_key,
                    index as i32,
                    tab_x,
                    tab_width,
                    local_x,
                    local_y,
                );
            } else {
                host.invoke_document_tab_pointer_clicked(
                    surface_key,
                    index as i32,
                    tab_x,
                    tab_width,
                    local_x,
                    local_y,
                );
            }
        }
        ChromePointerRoute::DrawerHeaderTab {
            surface_key,
            index,
            tab_x,
            tab_width,
            local_x,
            local_y,
        } => host.invoke_drawer_header_pointer_clicked(
            surface_key,
            index as i32,
            tab_x,
            tab_width,
            local_x,
            local_y,
        ),
        ChromePointerRoute::FloatingWindowHeader { .. } => {
            host.invoke_floating_window_header_pointer_clicked(x, y);
        }
        ChromePointerRoute::Resize => {
            arm_native_resize(ui, x, y);
        }
    }
}
