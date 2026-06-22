use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;

use super::super::binding_actions::{
    primary_change_action_id, primary_click_action_id, primary_click_binding_id,
    primary_submit_action_id,
};
use super::super::showcase_actions::{
    preferred_showcase_action_id, preferred_showcase_commit_action_id,
    preferred_showcase_drag_action_id, preferred_showcase_edit_action_id,
    preferred_showcase_pointer_drag_action_id,
};

pub(super) struct ProjectedActionIds {
    pub(super) dispatch_kind: String,
    pub(super) action_id: String,
    pub(super) binding_id: String,
    pub(super) begin_drag_action_id: String,
    pub(super) drag_action_id: String,
    pub(super) end_drag_action_id: String,
    pub(super) commit_action_id: String,
    pub(super) edit_action_id: String,
}

pub(super) fn projected_action_ids(
    control_id: &str,
    bindings: &[RetainedUiHostBindingProjection],
    component_descriptor: Option<&UiComponentDescriptor>,
    disabled: bool,
    popup_open: bool,
) -> ProjectedActionIds {
    let action_id = component_descriptor
        .and_then(|_| preferred_showcase_action_id(control_id, popup_open, bindings))
        .or_else(|| primary_click_action_id(bindings))
        .unwrap_or_default();

    ProjectedActionIds {
        dispatch_kind: if !disabled && !action_id.is_empty() {
            "showcase".to_string()
        } else {
            String::new()
        },
        action_id,
        binding_id: primary_click_binding_id(bindings).unwrap_or_default(),
        begin_drag_action_id: component_descriptor
            .and_then(|_| {
                preferred_showcase_pointer_drag_action_id(control_id, "DragBegin", bindings)
            })
            .unwrap_or_default(),
        drag_action_id: component_descriptor
            .and_then(|_| preferred_showcase_drag_action_id(control_id, bindings))
            .unwrap_or_default(),
        end_drag_action_id: component_descriptor
            .and_then(|_| {
                preferred_showcase_pointer_drag_action_id(control_id, "DragEnd", bindings)
            })
            .unwrap_or_default(),
        commit_action_id: component_descriptor
            .and_then(|_| preferred_showcase_commit_action_id(control_id, bindings))
            .or_else(|| primary_submit_action_id(bindings))
            .unwrap_or_default(),
        edit_action_id: component_descriptor
            .and_then(|_| preferred_showcase_edit_action_id(control_id, bindings))
            .or_else(|| primary_change_action_id(bindings))
            .unwrap_or_default(),
    }
}
