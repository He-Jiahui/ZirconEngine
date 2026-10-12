use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn dispatch_floating_window_header_press(ui: &UiHostWindow, x: f32, y: f32) {
    ui.global::<UiHostContext>()
        .invoke_floating_window_header_pointer_clicked(x, y);
}
