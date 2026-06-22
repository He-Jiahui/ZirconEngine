use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::menu_geometry::{
    menu_damage_frame, menu_handles_point, menu_popup_handles_point,
};

pub(super) fn dispatch_menu_pointer_scroll(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    delta: f32,
) -> Option<NativePointerDispatchResult> {
    if !menu_handles_point(presentation, x, y) && !menu_popup_handles_point(presentation, x, y) {
        return None;
    }
    ui.global::<UiHostContext>()
        .invoke_menu_pointer_scrolled(x, y, delta);
    Some(NativePointerDispatchResult::region(menu_damage_frame(
        presentation,
    )))
}
