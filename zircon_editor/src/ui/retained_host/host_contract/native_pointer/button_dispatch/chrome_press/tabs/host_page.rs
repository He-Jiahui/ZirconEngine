use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(in crate::ui::retained_host::host_contract::native_pointer::button_dispatch::chrome_press) fn dispatch_host_page_tab_press(
    ui: &UiHostWindow,
    index: usize,
    tab_x: f32,
    tab_width: f32,
    local_x: f32,
    local_y: f32,
) {
    ui.global::<UiHostContext>()
        .invoke_host_page_pointer_clicked(index as i32, tab_x, tab_width, local_x, local_y);
}
