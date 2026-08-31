use crate::ui::template_runtime::RetainedUiHostBindingProjection;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;

use super::super::binding_actions::binding_path_action_id;
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
    let primary = primary_binding_refs(bindings);
    let action_id = component_descriptor
        .and_then(|_| preferred_showcase_action_id(control_id, popup_open, bindings))
        .or_else(|| {
            primary.click.and_then(|binding| {
                (!binding.action_id.is_empty()).then(|| binding.action_id.clone())
            })
        })
        .unwrap_or_default();

    ProjectedActionIds {
        dispatch_kind: if !disabled && !action_id.is_empty() {
            "showcase".to_string()
        } else {
            String::new()
        },
        action_id,
        binding_id: primary
            .click
            .map(|binding| binding.binding_id.clone())
            .unwrap_or_default(),
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
            .or_else(|| {
                primary
                    .submit
                    .map(|binding| binding_path_action_id(&binding.binding_id))
            })
            .unwrap_or_default(),
        edit_action_id: component_descriptor
            .and_then(|_| preferred_showcase_edit_action_id(control_id, bindings))
            .or_else(|| {
                primary
                    .change
                    .map(|binding| binding_path_action_id(&binding.binding_id))
            })
            .unwrap_or_default(),
    }
}

#[derive(Default)]
struct PrimaryBindingRefs<'a> {
    click: Option<&'a RetainedUiHostBindingProjection>,
    change: Option<&'a RetainedUiHostBindingProjection>,
    submit: Option<&'a RetainedUiHostBindingProjection>,
}

fn primary_binding_refs(bindings: &[RetainedUiHostBindingProjection]) -> PrimaryBindingRefs<'_> {
    let mut primary = PrimaryBindingRefs::default();
    for binding in bindings {
        match binding.event_kind {
            UiEventKind::Click if primary.click.is_none() => primary.click = Some(binding),
            UiEventKind::Change if primary.change.is_none() => primary.change = Some(binding),
            UiEventKind::Submit if primary.submit.is_none() => primary.submit = Some(binding),
            _ => {}
        }
        if primary.click.is_some() && primary.change.is_some() && primary.submit.is_some() {
            break;
        }
    }
    primary
}

#[cfg(test)]
#[path = "action_ids/primary_binding_scan_tests.rs"]
mod primary_binding_scan_tests;
