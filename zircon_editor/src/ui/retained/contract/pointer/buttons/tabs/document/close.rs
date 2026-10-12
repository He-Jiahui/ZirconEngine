use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::primitives::SharedString;

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_document_tab_close_press(
    ui: &UiHostWindow,
    surface_key: SharedString,
    index: usize,
    tab_x: f32,
    tab_width: f32,
    local_x: f32,
    local_y: f32,
) {
    let host = ui.global::<UiHostContext>();
    host.invoke_document_tab_close_pointer_clicked(
        surface_key,
        index as i32,
        tab_x,
        tab_width,
        local_x,
        local_y,
    );
}
