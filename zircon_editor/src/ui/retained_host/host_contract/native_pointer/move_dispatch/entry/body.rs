use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::clear::clear_hovered_template_move;
use super::super::menu::dispatch_menu_pointer_move;
use super::super::page_overflow::dispatch_host_page_overflow_pointer_move;
use super::super::pane::dispatch_pane_pointer_move;
use super::super::workbench::{
    dispatch_workbench_template_pointer_move, dispatch_workbench_template_popup_pointer_move,
};

pub(super) fn dispatch_pointer_move_body(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let presentation = ui.get_host_presentation();
    let overflow = match dispatch_host_page_overflow_pointer_move(ui, &presentation, x, y) {
        Some(dispatch) if dispatch.consumed => return dispatch.result,
        other => other,
    };
    let routed = if let Some(result) = dispatch_menu_pointer_move(ui, &presentation, x, y) {
        result
    } else if let Some(result) =
        dispatch_workbench_template_popup_pointer_move(ui, &presentation, x, y)
    {
        result
    } else if let Some(result) = dispatch_pane_pointer_move(ui, &presentation, x, y) {
        result
    } else if let Some(result) = dispatch_workbench_template_pointer_move(ui, &presentation, x, y) {
        result
    } else {
        clear_hovered_template_move(ui)
    };
    match overflow {
        Some(dispatch) => dispatch.result.merge(routed),
        None => routed,
    }
}
