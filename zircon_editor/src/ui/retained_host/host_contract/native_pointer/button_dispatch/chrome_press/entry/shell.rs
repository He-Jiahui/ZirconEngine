use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::routing::ChromePointerRoute;
use super::super::{
    floating::dispatch_floating_window_header_press, rails::dispatch_activity_rail_press,
    resize::dispatch_resize_press,
};

pub(super) fn dispatch_chrome_shell_press(
    ui: &UiHostWindow,
    route: ChromePointerRoute,
    x: f32,
    y: f32,
) {
    match route {
        ChromePointerRoute::ActivityRail {
            side,
            control_id: _,
            local_x,
            local_y,
        } => dispatch_activity_rail_press(ui, side, local_x, local_y),
        ChromePointerRoute::FloatingWindowHeader { .. } => {
            dispatch_floating_window_header_press(ui, x, y);
        }
        ChromePointerRoute::Resize => dispatch_resize_press(ui, x, y),
        _ => {}
    }
}
