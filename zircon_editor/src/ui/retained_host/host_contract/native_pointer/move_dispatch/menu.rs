use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::menu_geometry::{
    menu_damage_frame, menu_handles_point, menu_popup_handles_point,
};

pub(super) fn dispatch_menu_pointer_move(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    if !menu_handles_point(presentation, x, y) && !menu_popup_handles_point(presentation, x, y) {
        return None;
    }
    let before = ui.get_menu_state();
    ui.global::<UiHostContext>().invoke_menu_pointer_moved(x, y);
    if before == ui.get_menu_state() {
        return Some(NativePointerDispatchResult::idle());
    }
    Some(NativePointerDispatchResult::region(menu_damage_frame(
        presentation,
    )))
}
