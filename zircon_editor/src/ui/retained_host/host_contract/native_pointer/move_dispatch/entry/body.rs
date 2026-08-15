use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::clear::clear_hovered_template_move;
use super::super::menu::dispatch_menu_pointer_move;
use super::super::page_overflow::dispatch_host_page_overflow_pointer_move;
use super::super::pane::dispatch_pane_pointer_move;
use super::super::workbench::{
    dispatch_workbench_template_hit, is_workbench_template_popup_hit,
    workbench_template_pointer_hit,
};

pub(super) fn dispatch_pointer_move_body(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let generation = ui.get_host_presentation_generation();
    let structure = generation.structure();
    let overflow = if generation.page_overflow_menu_state().open {
        dispatch_host_page_overflow_pointer_move(
            ui,
            structure,
            generation.page_overflow_menu_state(),
            x,
            y,
        )
    } else {
        None
    };
    let overflow = match overflow {
        Some(dispatch) if dispatch.consumed => return dispatch.result,
        other => other,
    };
    let routed = if let Some(result) = dispatch_menu_pointer_move(ui, &generation, x, y) {
        result
    } else {
        let workbench_hit = workbench_template_pointer_hit(&generation, x, y);
        if let Some(hit) = workbench_hit
            .as_ref()
            .filter(|hit| is_workbench_template_popup_hit(hit))
        {
            dispatch_workbench_template_hit(ui, hit)
        } else if let Some(result) = dispatch_pane_pointer_move(ui, structure, x, y) {
            result
        } else if let Some(hit) = workbench_hit.as_ref() {
            dispatch_workbench_template_hit(ui, hit)
        } else {
            clear_hovered_template_move(ui)
        }
    };
    match overflow {
        Some(dispatch) => dispatch.result.merge(routed),
        None => routed,
    }
}
