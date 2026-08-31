use crate::ui::retained_host::host_contract::data::{FrameRect, HostPresentationGeneration};
use crate::ui::retained_host::host_contract::native_popup_dismiss::dispatch_workbench_popup_outside_primary_press;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::chrome_route::dispatch_top_level_chrome_primary_press;
use super::dock_overflow_menu::dispatch_host_dock_overflow_menu_primary_press;
use super::menu_press::dispatch_menu_primary_press;
use super::page_overflow_menu::dispatch_host_page_overflow_menu_primary_press;

pub(super) fn dispatch_primary_press_overlays(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
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
    dispatch_top_level_chrome_primary_press(ui, structure, x, y, cleared_text_input_frame)
}
