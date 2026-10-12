#[path = "dispatch/actions.rs"]
mod actions;

use self::actions::{
    dispatch_popup_accept, dispatch_popup_cancel, dispatch_popup_hover_row,
    dispatch_popup_window_request,
};
use super::super::redraw::NativePointerDispatchResult;
use super::super::window::UiHostWindow;
use super::commands::{workbench_popup_keyboard_command, WorkbenchPopupKeyboardCommand};
use super::target::{active_popup_keyboard_target_for_ui, PopupKeyboardMove};
use winit::keyboard::Key;

pub(in crate::ui::retained_host::host_contract) fn workbench_popup_accept_is_owned(
    ui: &UiHostWindow,
    key: &Key,
) -> bool {
    matches!(
        workbench_popup_keyboard_command(key),
        Some(WorkbenchPopupKeyboardCommand::Accept)
    ) && active_popup_keyboard_target_for_ui(ui).is_some()
}

pub(in crate::ui::retained_host::host_contract) fn dispatch_workbench_popup_keyboard_command(
    ui: &UiHostWindow,
    command: WorkbenchPopupKeyboardCommand,
) -> NativePointerDispatchResult {
    let Some(target) = active_popup_keyboard_target_for_ui(ui) else {
        if matches!(command, WorkbenchPopupKeyboardCommand::Cancel) {
            let generation = ui.get_host_presentation_generation();
            if let Some(target) = crate::ui::retained_host::host_contract::native_popup_dismiss::unreal_viewport_overflow_dismiss_target(&generation.structure().workbench_window_nodes) {
                ui.global::<crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext>().invoke_surface_control_clicked(target.control_id, crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID.into());
                ui.clear_hovered_template_node_for_pointer_move();
                return NativePointerDispatchResult::region_with_frame_update(target.damage_frame);
            }
        }
        return NativePointerDispatchResult::idle();
    };

    match command {
        WorkbenchPopupKeyboardCommand::Next
        | WorkbenchPopupKeyboardCommand::Previous
        | WorkbenchPopupKeyboardCommand::First
        | WorkbenchPopupKeyboardCommand::Last
        | WorkbenchPopupKeyboardCommand::PageDown
        | WorkbenchPopupKeyboardCommand::PageUp => {
            let Some(next) = target.next_move(command) else {
                return NativePointerDispatchResult::idle();
            };
            match next {
                PopupKeyboardMove::Row(row) => dispatch_popup_hover_row(ui, target, row),
                PopupKeyboardMove::Window(request) => {
                    dispatch_popup_window_request(ui, target, request)
                }
            }
        }
        WorkbenchPopupKeyboardCommand::Accept => dispatch_popup_accept(ui, target),
        WorkbenchPopupKeyboardCommand::Cancel => dispatch_popup_cancel(ui, target),
    }
}

pub(in crate::ui::retained_host::host_contract) fn dispatch_workbench_popup_text_search(
    ui: &UiHostWindow,
    text: &str,
) -> NativePointerDispatchResult {
    let Some(target) = active_popup_keyboard_target_for_ui(ui) else {
        return NativePointerDispatchResult::idle();
    };
    let Some(next) = target.text_search_row(text) else {
        return NativePointerDispatchResult::idle();
    };
    dispatch_popup_hover_row(ui, target, next)
}
