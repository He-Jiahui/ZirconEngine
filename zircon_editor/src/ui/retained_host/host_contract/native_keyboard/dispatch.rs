mod actions;

use self::actions::{
    dispatch_popup_accept, dispatch_popup_cancel, dispatch_popup_hover_row,
    dispatch_popup_window_request,
};
use super::super::redraw::NativePointerDispatchResult;
use super::super::window::UiHostWindow;
use super::commands::WorkbenchPopupKeyboardCommand;
use super::target::{active_popup_keyboard_target_for_ui, PopupKeyboardMove};

pub(in crate::ui::retained_host::host_contract) fn dispatch_workbench_popup_keyboard_command(
    ui: &UiHostWindow,
    command: WorkbenchPopupKeyboardCommand,
) -> NativePointerDispatchResult {
    let Some(target) = active_popup_keyboard_target_for_ui(ui) else {
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
