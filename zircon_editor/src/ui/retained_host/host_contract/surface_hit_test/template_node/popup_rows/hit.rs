use crate::ui::retained_host::primitives::SharedString;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::template_component_family::template_component_family;
use super::super::TemplateNodePointerHit;

pub(in crate::ui::retained_host::host_contract) enum TemplatePopupRowHit {
    Hit(TemplateNodePointerHit),
    Blocked,
}

pub(super) fn template_popup_row_hit(
    node: &TemplatePaneNodeData,
    frame: FrameRect,
    dispatch_kind: &str,
    action_id: SharedString,
    value_text: SharedString,
) -> TemplateNodePointerHit {
    TemplateNodePointerHit {
        pane_id: SharedString::new(),
        control_id: node.control_id.clone(),
        action_id,
        binding_id: String::new(),
        dispatch_kind: dispatch_kind.to_string(),
        component_role: node.component_role.clone(),
        component_family: template_component_family(node),
        value_text,
        edit_action_id: node.edit_action_id.clone(),
        commit_action_id: node.commit_action_id.clone(),
        disabled: node.disabled,
        frame,
        table_row_source_index: None,
        table_row_identity_kind: SharedString::new(),
        table_row_identity_text: SharedString::new(),
    }
}
