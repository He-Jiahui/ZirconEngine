use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract::native_pointer::button_dispatch::chrome_press) fn dispatch_drawer_header_tab_press(
    ui: &UiHostWindow,
    surface_key: SharedString,
    index: usize,
    tab_x: f32,
    tab_width: f32,
    local_x: f32,
    local_y: f32,
) {
    ui.global::<UiHostContext>()
        .invoke_drawer_header_pointer_clicked(
            surface_key,
            index as i32,
            tab_x,
            tab_width,
            local_x,
            local_y,
        );
}
