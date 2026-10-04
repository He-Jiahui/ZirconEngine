use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiPointerId;

use super::super::super::routing::ChromePointerRoute;

mod shell;
mod tabs;

pub(in crate::ui::retained_host::host_contract) fn dispatch_chrome_press(
    ui: &UiHostWindow,
    pointer_id: UiPointerId,
    route: ChromePointerRoute,
    x: f32,
    y: f32,
) {
    if tabs::dispatch_chrome_tab_press(ui, &route) {
        return;
    }
    shell::dispatch_chrome_shell_press(ui, pointer_id, route, x, y);
}
