use crate::ui::retained_host::host_contract::data::HostPresentationGeneration;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::menu_geometry::{
    menu_damage_frame_with_state, menu_handles_point_with_state,
    menu_popup_handles_point_with_state,
};

pub(super) fn dispatch_menu_pointer_scroll(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
    delta: f32,
) -> Option<NativePointerDispatchResult> {
    let structure = generation.structure();
    let menu_state = generation.menu_state();
    let handles_menu_bar = menu_handles_point_with_state(structure, menu_state, x, y);
    let handles_popup = menu_popup_handles_point_with_state(structure, menu_state, x, y);
    if !handles_menu_bar && !handles_popup {
        return None;
    }
    let before = generation.interaction_generation();
    ui.global::<UiHostContext>()
        .invoke_menu_pointer_scrolled(x, y, delta);
    if before == ui.get_host_interaction_generation() {
        return Some(NativePointerDispatchResult::idle());
    }
    Some(NativePointerDispatchResult::region(
        menu_damage_frame_with_state(structure, menu_state),
    ))
}
