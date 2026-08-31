use super::dock_overflow::host_dock_overflow_keyboard_target_with_state;
use super::menu::menu_popup_keyboard_target;
use super::model::PopupKeyboardTarget;
use super::options::option_popup_keyboard_target;
use super::page_overflow::host_page_overflow_keyboard_target_with_state;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::template_geometry::template_popup_bounds;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::primitives::ModelRc;

pub(in crate::ui::retained_host::host_contract) fn active_popup_keyboard_target_for_ui(
    ui: &UiHostWindow,
) -> Option<PopupKeyboardTarget> {
    let generation = ui.get_host_presentation_generation();
    if generation.dock_overflow_menu_state().open {
        if let Some(target) = host_dock_overflow_keyboard_target_with_state(
            generation.structure(),
            generation.dock_overflow_menu_state(),
        ) {
            return Some(target);
        }
    }
    if generation.page_overflow_menu_state().open {
        if let Some(target) = host_page_overflow_keyboard_target_with_state(
            generation.structure(),
            generation.page_overflow_menu_state(),
        ) {
            return Some(target);
        }
    }
    let popup_rows = generation.workbench_hit_index().popup_rows();
    if popup_rows.is_empty() {
        return None;
    }
    let presentation = generation.structure();
    let interaction = generation.pane_interaction_state();
    let bounds = template_popup_bounds(
        &presentation.host_shell.native_window_bounds,
        &presentation.workbench_window_nodes,
    );
    active_popup_keyboard_target(
        &presentation.workbench_window_nodes,
        interaction,
        &bounds,
        popup_rows,
    )
}

fn active_popup_keyboard_target(
    nodes: &ModelRc<TemplatePaneNodeData>,
    interaction: &HostPaneInteractionStateData,
    bounds: &FrameRect,
    popup_rows: &[usize],
) -> Option<PopupKeyboardTarget> {
    let mut fallback = None;
    for row in popup_rows.iter().rev().copied() {
        let Some(node) = nodes.get(row) else {
            continue;
        };
        let Some(target) = popup_keyboard_target_for_node(node, interaction, bounds) else {
            continue;
        };
        let is_hovered_popup =
            node.control_id.as_str() == interaction.hovered_template_control_id.as_str();
        if is_hovered_popup || node.focused || node.selected || fallback.is_none() {
            fallback = Some(target);
        }
        if is_hovered_popup {
            break;
        }
    }
    fallback
}

fn popup_keyboard_target_for_node(
    node: &TemplatePaneNodeData,
    interaction: &HostPaneInteractionStateData,
    bounds: &FrameRect,
) -> Option<PopupKeyboardTarget> {
    if !node.popup_open || node.disabled || node.control_id.is_empty() {
        return None;
    }

    option_popup_keyboard_target(node, interaction, bounds)
        .or_else(|| menu_popup_keyboard_target(node, interaction))
}
