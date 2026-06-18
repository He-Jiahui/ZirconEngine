use super::super::frame_geometry::union_optional_frames;
use super::super::globals::PaneSurfaceHostContext;
use super::super::redraw::NativePointerDispatchResult;
use super::super::window::UiHostWindow;
use super::commands::WorkbenchPopupKeyboardCommand;
use super::target::{active_popup_keyboard_target_for_ui, PopupKeyboardRow, PopupKeyboardTarget};
use crate::ui::retained_host::callback_dispatch::{
    WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID, WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
};
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;

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

fn dispatch_popup_accept(
    ui: &UiHostWindow,
    target: PopupKeyboardTarget,
) -> NativePointerDispatchResult {
    let Some(row) = target.current_row.clone() else {
        return NativePointerDispatchResult::idle();
    };
    let popup_frame = target.popup_frame.clone();
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    match target.dispatch_kind.as_str() {
        "workbench_option"
            if target.control_id.as_str() == WORKBENCH_COMMAND_PALETTE_CONTROL_ID =>
        {
            pane_host.invoke_surface_control_edited(
                target.control_id,
                WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID.into(),
                row.value_text,
            )
        }
        "workbench_option" => pane_host.invoke_component_showcase_option_selected(
            target.control_id,
            row.action_id,
            row.value_text,
        ),
        "workbench_menu_item" => {
            pane_host.invoke_surface_control_clicked(target.control_id, row.action_id)
        }
        _ => return NativePointerDispatchResult::idle(),
    }
    NativePointerDispatchResult::region_with_frame_update(popup_frame)
}

fn dispatch_popup_cancel(
    ui: &UiHostWindow,
    target: PopupKeyboardTarget,
) -> NativePointerDispatchResult {
    let popup_frame = target.popup_frame.clone();
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    pane_host
        .invoke_surface_control_clicked(target.control_id, WORKBENCH_POPUP_CANCEL_ACTION_ID.into());
    ui.clear_hovered_template_node_for_pointer_move();
    NativePointerDispatchResult::region_with_frame_update(popup_frame)
}

fn dispatch_popup_hover_row(
    ui: &UiHostWindow,
    target: PopupKeyboardTarget,
    next: PopupKeyboardRow,
) -> NativePointerDispatchResult {
    ui.set_hovered_template_row_for_pointer_move(
        target.control_id.clone(),
        target.dispatch_kind,
        next.action_id,
        next.value_text,
        next.frame.clone(),
    );
    NativePointerDispatchResult::region(
        union_optional_frames(Some(target.current_frame), Some(next.frame)).unwrap_or_default(),
    )
}
