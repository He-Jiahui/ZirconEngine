use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{UiPointerComponentEvent, UiPointerComponentEventReason},
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeError, UiTreeNode},
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use crate::ui::tree::UiRuntimeTreeAccessExt;

use super::{
    bool_attribute_value, bool_component_state_value, is_default_radio_behavior,
    is_default_radio_group_behavior, widget_checked_property, UiDefaultKeyboardActionReport,
};

struct UiDefaultRadioMutation {
    radio_id: UiNodeId,
    checked_property: String,
    option_value: UiValue,
    sibling_unchecks: Vec<(UiNodeId, String)>,
    group: Option<UiDefaultRadioGroupMutation>,
}

struct UiDefaultRadioGroupMutation {
    node_id: UiNodeId,
    value_property: String,
}

struct UiDefaultRadioGroupLookup {
    node_id: UiNodeId,
    enabled: bool,
}

impl UiSurface {
    pub(super) fn apply_default_radio_component_action(
        &mut self,
        node_id: UiNodeId,
        events: &mut Vec<UiPointerComponentEvent>,
    ) -> Result<bool, UiTreeError> {
        let Some(mutation) = self.default_radio_mutation(node_id)? else {
            return Ok(false);
        };
        let radio_event = UiComponentEvent::ValueChanged {
            property: mutation.checked_property.clone(),
            value: UiValue::Bool(true),
        };
        let group_event = mutation
            .group
            .as_ref()
            .map(|group| UiComponentEvent::ValueChanged {
                property: group.value_property.clone(),
                value: mutation.option_value.clone(),
            });

        let report = self.apply_default_radio_mutation(&mutation)?;
        if !report.radio_changed {
            return Ok(false);
        }
        self.push_pointer_component_events(
            events,
            mutation.radio_id,
            UiEventKind::Change,
            radio_event,
            UiPointerComponentEventReason::DefaultClick,
        )?;
        if report.group_changed {
            if let (Some(group), Some(event)) = (mutation.group.as_ref(), group_event) {
                self.push_pointer_component_events(
                    events,
                    group.node_id,
                    UiEventKind::Change,
                    event,
                    UiPointerComponentEventReason::DefaultClick,
                )?;
            }
        }
        Ok(true)
    }

    pub(super) fn apply_default_radio_keyboard_action(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let Some(mutation) = self.default_radio_mutation(node_id)? else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };
        let radio_event = UiComponentEvent::ValueChanged {
            property: mutation.checked_property.clone(),
            value: UiValue::Bool(true),
        };
        let group_event = mutation
            .group
            .as_ref()
            .map(|group| UiComponentEvent::ValueChanged {
                property: group.value_property.clone(),
                value: mutation.option_value.clone(),
            });

        let report = self.apply_default_radio_mutation(&mutation)?;
        if !report.radio_changed {
            return Ok(UiDefaultKeyboardActionReport::default());
        }

        let mut component_events = self.component_event_reports_for_bindings(
            mutation.radio_id,
            UiEventKind::Change,
            radio_event,
            true,
        )?;
        if report.group_changed {
            if let (Some(group), Some(event)) = (mutation.group.as_ref(), group_event) {
                component_events.extend(self.component_event_reports_for_bindings(
                    group.node_id,
                    UiEventKind::Change,
                    event,
                    true,
                )?);
            }
        }
        Ok(UiDefaultKeyboardActionReport {
            handled: true,
            component_events,
        })
    }

    fn default_radio_mutation(
        &self,
        node_id: UiNodeId,
    ) -> Result<Option<UiDefaultRadioMutation>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata)
            || !is_default_radio_behavior(metadata)
        {
            return Ok(None);
        }
        let checked_property = widget_checked_property(metadata).to_string();
        if self.default_radio_current_checked(node_id, node, metadata, &checked_property) {
            return Ok(None);
        }
        let option_value = default_radio_option_value(node, metadata);
        let group_lookup = self.default_radio_group(node_id)?;
        if matches!(group_lookup.as_ref(), Some(group) if !group.enabled) {
            return Ok(None);
        }
        let group = group_lookup.map(|group| {
            let value_property = self
                .tree
                .node(group.node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .map(widget_value_property)
                .unwrap_or("value")
                .to_string();
            UiDefaultRadioGroupMutation {
                node_id: group.node_id,
                value_property,
            }
        });
        let sibling_unchecks = match group.as_ref() {
            Some(group) => self.checked_radio_descendants(group.node_id, node_id)?,
            None => Vec::new(),
        };

        Ok(Some(UiDefaultRadioMutation {
            radio_id: node_id,
            checked_property,
            option_value,
            sibling_unchecks,
            group,
        }))
    }

    fn apply_default_radio_mutation(
        &mut self,
        mutation: &UiDefaultRadioMutation,
    ) -> Result<UiDefaultRadioMutationReport, UiTreeError> {
        for (sibling_id, property) in &mutation.sibling_unchecks {
            self.mutate_property(UiPropertyMutationRequest::new(
                *sibling_id,
                property.clone(),
                UiValue::Bool(false),
            ))?;
        }

        let radio_report = self.mutate_property(UiPropertyMutationRequest::new(
            mutation.radio_id,
            mutation.checked_property.clone(),
            UiValue::Bool(true),
        ))?;
        if !matches!(radio_report.status, UiPropertyMutationStatus::Accepted) {
            return Ok(UiDefaultRadioMutationReport {
                radio_changed: false,
                group_changed: false,
            });
        }

        let group_changed = if let Some(group) = mutation.group.as_ref() {
            let group_report = self.mutate_property(UiPropertyMutationRequest::new(
                group.node_id,
                group.value_property.clone(),
                mutation.option_value.clone(),
            ))?;
            matches!(group_report.status, UiPropertyMutationStatus::Accepted)
        } else {
            false
        };

        Ok(UiDefaultRadioMutationReport {
            radio_changed: true,
            group_changed,
        })
    }

    fn default_radio_group(
        &self,
        node_id: UiNodeId,
    ) -> Result<Option<UiDefaultRadioGroupLookup>, UiTreeError> {
        let mut current = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?
            .parent;
        while let Some(candidate_id) = current {
            let candidate = self
                .tree
                .node(candidate_id)
                .ok_or(UiTreeError::MissingNode(candidate_id))?;
            if let Some(metadata) = candidate.template_metadata.as_ref() {
                if is_default_radio_group_behavior(metadata) {
                    return Ok(Some(UiDefaultRadioGroupLookup {
                        node_id: candidate_id,
                        enabled: self.widget_interaction_enabled(candidate_id, candidate, metadata),
                    }));
                }
            }
            current = candidate.parent;
        }
        Ok(None)
    }

    fn checked_radio_descendants(
        &self,
        root_id: UiNodeId,
        selected_id: UiNodeId,
    ) -> Result<Vec<(UiNodeId, String)>, UiTreeError> {
        let mut checked = Vec::new();
        self.collect_checked_radio_descendants(root_id, selected_id, &mut checked)?;
        Ok(checked)
    }

    fn collect_checked_radio_descendants(
        &self,
        root_id: UiNodeId,
        selected_id: UiNodeId,
        checked: &mut Vec<(UiNodeId, String)>,
    ) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .node(root_id)
            .ok_or(UiTreeError::MissingNode(root_id))?;
        for child_id in &node.children {
            let child = self
                .tree
                .node(*child_id)
                .ok_or(UiTreeError::MissingNode(*child_id))?;
            if *child_id != selected_id {
                if let Some(metadata) = child.template_metadata.as_ref() {
                    if is_default_radio_behavior(metadata) {
                        let property = widget_checked_property(metadata);
                        if self.default_radio_current_checked(*child_id, child, metadata, property)
                        {
                            checked.push((*child_id, property.to_string()));
                        }
                    }
                }
            }
            self.collect_checked_radio_descendants(*child_id, selected_id, checked)?;
        }
        Ok(())
    }

    fn default_radio_current_checked(
        &self,
        node_id: UiNodeId,
        node: &UiTreeNode,
        metadata: &UiTemplateNodeMetadata,
        property: &str,
    ) -> bool {
        let component_state = self.component_states.get(node_id);
        bool_attribute_value(&metadata.attributes, property)
            .or_else(|| {
                component_state.and_then(|state| bool_component_state_value(state, property))
            })
            .or_else(|| component_state.and_then(|state| state.flags.checked.then_some(true)))
            .unwrap_or(node.state_flags.checked || metadata.widget.checked.unwrap_or(false))
    }
}

struct UiDefaultRadioMutationReport {
    radio_changed: bool,
    group_changed: bool,
}

fn default_radio_option_value(node: &UiTreeNode, metadata: &UiTemplateNodeMetadata) -> UiValue {
    metadata.widget.value.clone().unwrap_or_else(|| {
        UiValue::String(
            metadata
                .control_id
                .clone()
                .unwrap_or_else(|| node.node_path.0.clone()),
        )
    })
}

fn widget_value_property(metadata: &UiTemplateNodeMetadata) -> &str {
    metadata.widget.value_property.as_deref().unwrap_or("value")
}
