use crate::ui::retained_host::host_contract::data::{FrameRect, HostPresentationGeneration};
use crate::ui::retained_host::host_contract::native_pointer::routing::route_pointer_to_workbench_generation;
use crate::ui::retained_host::host_contract::native_pointer::NativePointerButtonState;
use crate::ui::retained_host::host_contract::native_popup_dismiss::dispatch_workbench_popup_outside_primary_press;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiPointerId;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::chrome_route::dispatch_top_level_chrome_primary_press;
use super::dock_overflow_menu::dispatch_host_dock_overflow_menu_primary_press;
use super::menu_press::dispatch_menu_primary_press;
use super::page_overflow_menu::dispatch_host_page_overflow_menu_primary_press;
use super::workbench::dispatch_workbench_button;

pub(super) fn dispatch_primary_press_overlays(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    pointer_id: UiPointerId,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    let structure = generation.structure();
    if let Some(result) = dispatch_host_dock_overflow_menu_primary_press(
        ui,
        structure,
        generation.dock_overflow_menu_state(),
        x,
        y,
        cleared_text_input_frame.clone(),
    ) {
        return Some(result);
    }
    if let Some(result) = dispatch_host_page_overflow_menu_primary_press(
        ui,
        structure,
        generation.page_overflow_menu_state(),
        x,
        y,
        cleared_text_input_frame.clone(),
    ) {
        return Some(result);
    }
    // The mounted authored menu owns its native press before legacy chrome.
    // Resolve the current generation again through the existing button owner.
    let authored_menu_hit = route_pointer_to_workbench_generation(generation, x, y);
    if authored_menu_hit.as_ref().is_some_and(|hit| {
        matches!(
            (hit.control_id.as_str(), hit.action_id.as_str()),
            ("MenuSlot0", "workbench.unreal.menu.file.open")
                | ("MenuSlot1", "workbench.unreal.menu.edit.open")
                | ("MenuSlot2", "workbench.unreal.menu.window.open")
                | ("MenuSlot3", "workbench.unreal.menu.tools.open")
                | ("MenuSlot4", "workbench.unreal.menu.build.open")
                | ("MenuSlot5", "workbench.unreal.menu.help.open")
        )
    }) {
        if let Some(result) = dispatch_workbench_button(
            ui,
            generation,
            NativePointerButtonState::Pressed,
            UiPointerButton::Primary,
            x,
            y,
            cleared_text_input_frame.clone(),
        ) {
            return Some(result);
        }
    }
    if let Some(result) = dispatch_menu_primary_press(
        ui,
        structure,
        generation.menu_state(),
        x,
        y,
        cleared_text_input_frame.clone(),
    ) {
        return Some(result);
    }
    if let Some(result) = dispatch_workbench_popup_outside_primary_press(
        ui,
        generation,
        x,
        y,
        cleared_text_input_frame.clone(),
    ) {
        return Some(result);
    }
    dispatch_top_level_chrome_primary_press(
        ui,
        structure,
        pointer_id,
        x,
        y,
        cleared_text_input_frame,
    )
}
