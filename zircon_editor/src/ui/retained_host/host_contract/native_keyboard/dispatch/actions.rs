use super::super::super::frame_geometry::union_optional_frames;
use super::super::super::globals::PaneSurfaceHostContext;
use super::super::super::redraw::NativePointerDispatchResult;
use super::super::super::window::UiHostWindow;
use super::super::target::{PopupKeyboardRow, PopupKeyboardTarget};
use crate::ui::retained_host::callback_dispatch::{
    WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID, WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
};
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;

pub(super) fn dispatch_popup_accept(
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

pub(super) fn dispatch_popup_cancel(
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

pub(super) fn dispatch_popup_hover_row(
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
