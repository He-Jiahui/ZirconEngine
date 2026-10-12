use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn dispatch_activity_rail_press(
    ui: &UiHostWindow,
    side: SharedString,
    local_x: f32,
    local_y: f32,
) {
    ui.global::<UiHostContext>()
        .invoke_activity_rail_pointer_clicked(side, local_x, local_y);
}
