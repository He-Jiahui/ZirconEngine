use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;

use super::super::drag_overlay::ProjectedDragOverlayData;
use super::super::popup_actions::ProjectedPopupActions;
use super::super::string_lists::to_host_contract_shared_string_list;

pub(super) fn assign_interaction_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    popup_actions: ProjectedPopupActions,
    drag_overlay: ProjectedDragOverlayData,
) {
    node.popup_open = popup_actions.popup_open;
    node.has_popup_anchor = popup_actions.has_popup_anchor;
    node.popup_anchor_x = popup_actions.popup_anchor_x;
    node.popup_anchor_y = popup_actions.popup_anchor_y;
    node.menu_items = to_host_contract_shared_string_list(popup_actions.menu_items);
    node.structured_menu_items = model_rc(popup_actions.structured_menu_items);
    node.actions = model_rc(popup_actions.actions);
    node.accepted_drag_payloads = popup_actions.accepted_drag_payloads.into();
    node.dispatch_kind = popup_actions.dispatch_kind.into();
    node.action_id = popup_actions.action_id.into();
    node.binding_id = popup_actions.binding_id.into();
    node.begin_drag_action_id = popup_actions.begin_drag_action_id.into();
    node.drag_action_id = popup_actions.drag_action_id.into();
    node.end_drag_action_id = popup_actions.end_drag_action_id.into();
    node.commit_action_id = popup_actions.commit_action_id.into();
    node.edit_action_id = popup_actions.edit_action_id.into();
    node.frame = popup_actions.frame;

    node.drop_source_summary = drag_overlay.drop_source_summary.into();
    node.drag_payload_kind = drag_overlay.payload_kind.into();
    node.drag_payload_label = drag_overlay.payload_label.into();
    node.drag_payload_reference = drag_overlay.payload_reference.into();
    node.has_drag_cursor = drag_overlay.has_cursor;
    node.drag_cursor_x = drag_overlay.cursor_x;
    node.drag_cursor_y = drag_overlay.cursor_y;
    node.drag_offset_x = drag_overlay.offset_x;
    node.drag_offset_y = drag_overlay.offset_y;
    node.drag_preview_width = drag_overlay.preview_width;
    node.drag_preview_height = drag_overlay.preview_height;
    node.drop_allowed = drag_overlay.drop_allowed;
    node.has_drop_target = drag_overlay.has_drop_target;
    node.drop_target_x = drag_overlay.drop_target_x;
    node.drop_target_y = drag_overlay.drop_target_y;
    node.drop_target_width = drag_overlay.drop_target_width;
    node.drop_target_height = drag_overlay.drop_target_height;
    node.drop_indicator_edge = drag_overlay.drop_indicator_edge.into();
    node.drop_indicator_text = drag_overlay.drop_indicator_text.into();
}
