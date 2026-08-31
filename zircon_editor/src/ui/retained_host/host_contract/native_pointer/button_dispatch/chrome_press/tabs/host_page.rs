use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(in crate::ui::retained_host::host_contract::native_pointer::button_dispatch::chrome_press) fn dispatch_host_page_tab_press(
    ui: &UiHostWindow,
    index: i32,
    close: bool,
) {
    ui.global::<UiHostContext>()
        .invoke_host_page_pointer_clicked(index, close);
}
