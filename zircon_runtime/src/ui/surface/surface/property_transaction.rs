use crate::ui::surface::{
    component_state::{UiComponentStatePropertyChange, property_may_affect_runtime_pseudo_state},
    property_mutation::{
        UiPropertyMutationReport, UiPropertyMutationRequest, UiPropertyMutationStatus,
        mutate_tree_property,
    },
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::UiTransientDismissalTarget,
    event_ui::UiNodeId,
    focus::{UiFocusChangeEvent, UiFocusChangeReason},
    surface::UiTextCaretAffinity,
    tree::{UiDirtyFlags, UiTree, UiTreeError},
};

use super::UiSurface;

impl UiSurface {
    pub fn mutate_property(
        &mut self,
        request: UiPropertyMutationRequest,
    ) -> Result<UiPropertyMutationReport, UiTreeError> {
        self.mutate_property_with_popup_branch_close(request, true)
    }

    pub(crate) fn dismiss_transient_ui(
        &mut self,
        target: UiTransientDismissalTarget,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        let popup_closures = matches!(
            target,
            UiTransientDismissalTarget::All | UiTransientDismissalTarget::PopupStack
        )
        .then(|| self.declarative_popup_closures())
        .unwrap_or_default();
        let route_owner = self.input.dismiss_transient_ui(target);
        for (popup_node_id, property) in popup_closures {
            let _ = self.mutate_property_with_popup_branch_close(
                UiPropertyMutationRequest::new(popup_node_id, property, UiValue::Bool(false)),
                false,
            )?;
        }
        Ok(route_owner)
    }

    pub(crate) fn dismiss_popup_by_id(
        &mut self,
        popup_id: &str,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        let fallback_route_owner = self.input.popup_owner(popup_id);
        let popup_node_id = self
            .input
            .popup_stack
            .iter()
            .find(|popup| popup.popup_id == popup_id)
            .and_then(|popup| popup.popup_node)
            .or_else(|| {
                self.unique_popup_state_for_id(popup_id)
                    .map(|(node_id, _, _)| node_id)
            });
        let Some(popup_node_id) = popup_node_id else {
            self.input.close_popup(popup_id);
            return Ok(fallback_route_owner);
        };
        let route_owner = self.popup_route_owner_for_node(popup_node_id).or_else(|| {
            (!self.is_popup_stack_node(popup_node_id))
                .then_some(fallback_route_owner)
                .flatten()
        });
        let mut popup_closures = self.popup_branch_closures(popup_node_id);
        let Some((property, open)) = self.popup_state_for_node(popup_node_id) else {
            self.input.close_popup_with_node(popup_node_id, popup_id);
            return Ok(route_owner);
        };
        if !open {
            self.input.close_popup_with_node(popup_node_id, popup_id);
            return Ok(route_owner);
        }
        popup_closures.push((popup_node_id, property.to_string()));
        for (popup_node_id, property) in popup_closures {
            let _ = self.mutate_property_with_popup_branch_close(
                UiPropertyMutationRequest::new(popup_node_id, property, UiValue::Bool(false)),
                false,
            )?;
        }
        Ok(route_owner)
    }

    pub(crate) fn set_declarative_popup_open_by_id(
        &mut self,
        popup_id: &str,
        open: bool,
    ) -> Result<bool, UiTreeError> {
        let Some((popup_node_id, property, current_open)) =
            self.unique_popup_state_for_id(popup_id)
        else {
            return Ok(false);
        };
        if current_open == open {
            if open {
                self.synchronize_open_popup_state(popup_node_id, property)?;
            }
            return Ok(true);
        }
        let _ = self.mutate_property(UiPropertyMutationRequest::new(
            popup_node_id,
            property,
            UiValue::Bool(open),
        ))?;
        Ok(true)
    }

    fn mutate_property_with_popup_branch_close(
        &mut self,
        request: UiPropertyMutationRequest,
        close_popup_branch: bool,
    ) -> Result<UiPropertyMutationReport, UiTreeError> {
        if let Some(report) = self.mutate_editable_text_property(&request) {
            return Ok(report);
        }
        let node_id = request.node_id;
        let property = request.property.clone();
        let value = request.value.clone();
        let tracks_text_edit_revision = text_edit_property_is_tracked(self, node_id, &property);
        let popup_close_descendants = (close_popup_branch
            && matches!(property.as_str(), "open" | "popup_open")
            && matches!(&value, UiValue::Bool(false))
            && self.is_popup_stack_node(node_id))
        .then(|| self.popup_branch_closures(node_id))
        .unwrap_or_default();
        for (descendant_id, descendant_property) in popup_close_descendants {
            let _ = self.mutate_property_with_popup_branch_close(
                UiPropertyMutationRequest::new(
                    descendant_id,
                    descendant_property,
                    UiValue::Bool(false),
                ),
                false,
            )?;
        }
        let mut report = mutate_tree_property(&mut self.tree, request)?;
        let previous_component_value =
            if matches!(report.status, UiPropertyMutationStatus::Accepted) {
                self.component_states
                    .get(node_id)
                    .and_then(|state| state.value(&property).cloned())
            } else {
                None
            };
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            if let Some(attribute_value) = self
                .tree
                .nodes
                .get(&node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .and_then(|metadata| metadata.attributes.get(&property))
                .cloned()
            {
                let _ = self.runtime_style.set_base_attribute(
                    node_id,
                    property.clone(),
                    attribute_value,
                );
            }
        }
        let component_state_change = if matches!(report.status, UiPropertyMutationStatus::Accepted)
        {
            self.component_states
                .sync_from_property(node_id, &property, &value)
        } else {
            UiComponentStatePropertyChange::default()
        };
        let popup_open_alias_state_change =
            if matches!(report.status, UiPropertyMutationStatus::Accepted) {
                self.sync_popup_open_alias_state(node_id, &property, &value)
            } else {
                UiComponentStatePropertyChange::default()
            };
        let component_state_change = component_state_change.merge(popup_open_alias_state_change);
        let custom_pseudo_state_changed = matches!(&value, UiValue::Bool(_))
            && !property_may_affect_runtime_pseudo_state(&property)
            && self.runtime_style.depends_on_pseudo_state(&property);
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            if component_state_change.pseudo_state_changed || custom_pseudo_state_changed {
                self.mark_component_state_render_dirty(node_id)?;
                report.mark_render_dirty();
            } else if component_state_change.value_changed {
                self.mark_node_dirty(
                    node_id,
                    UiDirtyFlags {
                        render: true,
                        ..UiDirtyFlags::default()
                    },
                )?;
                report.mark_render_dirty();
            } else if property_may_affect_runtime_pseudo_state(&property) {
                let changed = self.apply_runtime_state_style_subtree(node_id, true)?;
                if changed > 0 {
                    report.mark_render_dirty();
                }
            }
            if component_state_change.any_changed() {
                report.record_component_state_value_update(
                    node_id,
                    property.clone(),
                    previous_component_value,
                    value.clone(),
                );
            }
        }
        if matches!(report.status, UiPropertyMutationStatus::Accepted)
            && matches!(
                property.as_str(),
                "disabled" | "enabled" | "visible" | "visibility" | "focusable"
            )
        {
            let reason = focus_reconcile_reason(&property, &self.tree, node_id);
            report.focus_change = self.reconcile_focus_after_tree_change(reason);
        }
        if matches!(report.status, UiPropertyMutationStatus::Accepted)
            && matches!(property.as_str(), "open" | "popup_open")
        {
            if let UiValue::Bool(open) = value {
                let popup_stack_node = self.is_popup_stack_node(node_id);
                if !open && popup_stack_node {
                    self.reset_popup_open_state(node_id, property.as_str())?;
                }
                if open {
                    if self.synchronize_open_popup_state(node_id, property.as_str())? {
                        report.mark_render_dirty();
                    }
                } else {
                    let runtime_anchored_popup = self.popup_uses_runtime_anchor(node_id);
                    let popup_owner = self.sync_popup_stack_for_node(node_id, false);
                    report.focus_change = self.apply_mui_modal_focus_transition(
                        node_id,
                        false,
                        runtime_anchored_popup.then_some(popup_owner).flatten(),
                    )?;
                }
            }
        }
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            self.invalidation
                .record_dirty(node_id, report.invalidation.dirty);
            if tracks_text_edit_revision {
                self.invalidate_clipboard_transfers_for(node_id);
            }
        }
        Ok(report)
    }

    fn mutate_editable_text_property(
        &mut self,
        request: &UiPropertyMutationRequest,
    ) -> Option<UiPropertyMutationReport> {
        if !super::input::is_editable_text_input(self, request.node_id) {
            return None;
        }
        if super::input::is_editable_text_derived_property(request.property.as_str())
            || super::input::is_number_field_internal_property(
                self,
                request.node_id,
                request.property.as_str(),
            )
        {
            return Some(UiPropertyMutationReport::rejected(
                request,
                format!(
                    "editable text state property '{}' requires an editable text transaction",
                    request.property
                ),
            ));
        }

        let value_property = super::input::editable_value_property(self, request.node_id)?;
        if request.property != value_property {
            return None;
        }
        let previous = self
            .tree
            .node(request.node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get(value_property.as_str()))
            .map(UiValue::from_toml);
        let Some(mut state) = super::input::editable_text_state_for_node(self, request.node_id)
        else {
            return Some(UiPropertyMutationReport::rejected(
                request,
                "editable text transaction could not resolve retained edit state",
            ));
        };
        let next_text = request.value.display_text();
        if state.text != next_text {
            let previous_caret = state.caret.offset;
            state.text = next_text;
            state.caret.offset =
                crate::ui::text::clamp_grapheme_boundary(state.text.as_str(), previous_caret);
            if state.caret.offset != previous_caret {
                state.caret.affinity = UiTextCaretAffinity::Downstream;
            }
            state.selection = None;
            state.composition = None;
        }

        match super::input::commit_editable_text_properties_with_value(
            self,
            request.node_id,
            value_property.as_str(),
            request.value.clone(),
            &state,
            request.effective_binding_source_kind(),
        ) {
            Ok(receipt) if receipt.changed_properties.is_empty() => {
                Some(UiPropertyMutationReport::unchanged(request, previous))
            }
            Ok(receipt) => {
                if receipt.value_changed {
                    self.input.advance_text_document_epoch(request.node_id);
                }
                let mut report =
                    UiPropertyMutationReport::accepted(request, previous, receipt.dirty);
                report.binding = receipt.binding_report.unwrap_or_default();
                Some(report)
            }
            Err(error) => Some(UiPropertyMutationReport::rejected(
                request,
                format!(
                    "editable text transaction rejected: {}",
                    error.diagnostic_code()
                ),
            )),
        }
    }

    fn synchronize_open_popup_state(
        &mut self,
        node_id: UiNodeId,
        property: &str,
    ) -> Result<bool, UiTreeError> {
        let runtime_anchored_popup = self.popup_uses_runtime_anchor(node_id);
        let popup_owner = self.sync_popup_stack_for_node(node_id, true);
        if runtime_anchored_popup && popup_owner.is_none() {
            let _ = self.reject_runtime_anchored_popup(node_id, property)?;
            return Ok(true);
        }
        let _ = self.apply_mui_modal_focus_transition(
            node_id,
            true,
            runtime_anchored_popup.then_some(popup_owner).flatten(),
        )?;
        Ok(false)
    }

    fn sync_popup_open_alias_state(
        &mut self,
        node_id: UiNodeId,
        property: &str,
        value: &UiValue,
    ) -> UiComponentStatePropertyChange {
        if !matches!(value, UiValue::Bool(_)) || !self.is_popup_stack_node(node_id) {
            return UiComponentStatePropertyChange::default();
        }
        let alias = match property {
            "open" => "popup_open",
            "popup_open" => "open",
            _ => return UiComponentStatePropertyChange::default(),
        };
        let Some(attribute_value) = self
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get(alias))
            .cloned()
        else {
            return UiComponentStatePropertyChange::default();
        };
        let _ = self
            .runtime_style
            .set_base_attribute(node_id, alias.to_string(), attribute_value);
        self.component_states
            .sync_from_property(node_id, alias, value)
    }

    pub(crate) fn reject_runtime_anchored_popup(
        &mut self,
        node_id: UiNodeId,
        property: &str,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        self.reset_popup_open_state(node_id, property)?;
        let _ = self.sync_popup_stack_for_node(node_id, false);
        self.apply_mui_modal_focus_transition(node_id, false, None)
    }

    fn reset_popup_open_state(
        &mut self,
        node_id: UiNodeId,
        property: &str,
    ) -> Result<(), UiTreeError> {
        let value = UiValue::Bool(false);
        let properties = if let Some(metadata) = self
            .tree
            .nodes
            .get_mut(&node_id)
            .and_then(|node| node.template_metadata.as_mut())
        {
            let properties = ["open", "popup_open"]
                .into_iter()
                .filter(|candidate| {
                    *candidate == property || metadata.attributes.contains_key(*candidate)
                })
                .collect::<Vec<_>>();
            for property in &properties {
                metadata
                    .attributes
                    .insert((*property).to_string(), toml::Value::Boolean(false));
            }
            properties
        } else {
            Vec::new()
        };
        if properties.is_empty() {
            return Ok(());
        }
        let mut component_state_change = UiComponentStatePropertyChange::default();
        for property in properties {
            let _ = self.runtime_style.set_base_attribute(
                node_id,
                property.to_string(),
                toml::Value::Boolean(false),
            );
            component_state_change = component_state_change.merge(
                self.component_states
                    .sync_from_property(node_id, property, &value),
            );
        }
        if component_state_change.pseudo_state_changed {
            self.mark_component_state_render_dirty(node_id)?;
        }
        self.mark_node_dirty(
            node_id,
            UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            },
        )?;
        Ok(())
    }
}

fn text_edit_property_is_tracked(surface: &UiSurface, node_id: UiNodeId, property: &str) -> bool {
    if !surface.has_pending_clipboard_transfer(node_id)
        || !super::input::is_editable_text_input(surface, node_id)
    {
        return false;
    }
    super::input::editable_value_property(surface, node_id).as_deref() == Some(property)
        || super::input::is_editable_text_derived_property(property)
        || matches!(
            property,
            "read_only"
                | "readOnly"
                | "input_read_only"
                | "inputReadOnly"
                | "secure"
                | "password"
                | "input_kind"
                | "inputKind"
        )
}

fn focus_reconcile_reason(property: &str, tree: &UiTree, node_id: UiNodeId) -> UiFocusChangeReason {
    match property {
        "disabled" | "enabled" | "focusable" => UiFocusChangeReason::Disabled,
        "visible" => UiFocusChangeReason::Hidden,
        "visibility" => tree
            .nodes
            .get(&node_id)
            .map(|node| {
                if node.is_render_visible() {
                    UiFocusChangeReason::Disabled
                } else {
                    UiFocusChangeReason::Hidden
                }
            })
            .unwrap_or(UiFocusChangeReason::Hidden),
        _ => UiFocusChangeReason::Clear,
    }
}
