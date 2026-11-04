use super::dispatch::actions::{dispatch_popup_accept, dispatch_popup_cancel_control};
use super::target::active_popup_keyboard_target_for_ui;
use crate::ui::retained_host::callback_dispatch::{
    WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID, WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
};
use crate::ui::retained_host::host_contract::data::HostTextInputFocusData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::template_geometry::frame_from_template_node;
use crate::ui::retained_host::host_contract::template_popup_layout::is_command_palette;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use winit::keyboard::NamedKey;

/// Search text owns editing, while Enter/Escape keep the palette's candidate/cancel semantics.
pub(in crate::ui::retained_host::host_contract) fn dispatch_focused_command_palette_key(
    ui: &UiHostWindow,
    focus: &HostTextInputFocusData,
    key: NamedKey,
) -> Option<NativePointerDispatchResult> {
    if focus.control_id.as_str() != WORKBENCH_COMMAND_PALETTE_CONTROL_ID
        || focus.edit_action_id.as_str() != "CommandPalette/QueryChanged"
        || focus.commit_action_id.as_str() != WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID
    {
        return None;
    }
    match key {
        NamedKey::Enter => {
            let target = active_popup_keyboard_target_for_ui(ui).filter(|target| {
                target.control_id == focus.control_id
                    && target.dispatch_kind.as_str() == "workbench_option"
            });
            Some(
                target.map_or_else(NativePointerDispatchResult::idle, |target| {
                    ui.global::<UiHostContext>().clear_text_input_focus();
                    dispatch_popup_accept(ui, target)
                }),
            )
        }
        NamedKey::Escape => {
            let generation = ui.get_host_presentation_generation();
            let target = generation
                .structure()
                .workbench_window_nodes
                .iter()
                .find(|node| {
                    node.control_id == focus.control_id
                        && is_command_palette(node)
                        && node.popup_open
                        && !node.disabled
                })
                .map(|node| (node.control_id.clone(), frame_from_template_node(node)));
            drop(generation);
            Some(
                target.map_or_else(NativePointerDispatchResult::idle, |(control_id, frame)| {
                    dispatch_popup_cancel_control(ui, control_id, frame)
                }),
            )
        }
        _ => None,
    }
}
