mod actions;

use self::actions::{dispatch_popup_accept, dispatch_popup_cancel, dispatch_popup_hover_row};
use super::super::redraw::NativePointerDispatchResult;
use super::super::window::UiHostWindow;
use super::commands::WorkbenchPopupKeyboardCommand;
use super::target::active_popup_keyboard_target_for_ui;

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
        | WorkbenchPopupKeyboardCommand::Last => {
            let Some(next) = target.next_row(command) else {
                return NativePointerDispatchResult::idle();
            };
            dispatch_popup_hover_row(ui, target, next)
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
