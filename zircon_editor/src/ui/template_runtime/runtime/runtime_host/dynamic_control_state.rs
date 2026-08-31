use std::collections::BTreeMap;

use crate::ui::layouts::windows::workbench_host_window::PaneBodyPresentation;
use crate::ui::template_runtime::{RetainedUiHostModel, RetainedUiProjection};
use zircon_runtime::ui::surface::{UiPropertyMutationRequest, UiSurface};
use zircon_runtime_interface::ui::{
    component::UiValue, dispatch::UiTemplateActionInvocation, event_ui::UiNodeId,
};

use super::super::pane_payload_projection::{
    append_hybrid_slot_anchor_projection, inject_pane_projection_attributes,
    template_v2_component_patch_attributes,
};
use super::{EditorUiHostRuntime, EditorUiHostRuntimeError};

impl EditorUiHostRuntime {
    pub(crate) fn project_pane_body(
        &self,
        body: &PaneBodyPresentation,
    ) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
        let mut projection = self.project_document_cached(&body.document_id)?;
        let pane_attributes = inject_pane_projection_attributes(&mut projection.root, body);
        append_hybrid_slot_anchor_projection(&mut projection.root, body, pane_attributes);
        Ok(projection)
    }

    pub(crate) fn apply_pane_component_patches_to_surface(
        &self,
        body: &PaneBodyPresentation,
        surface: &mut UiSurface,
    ) -> Result<(), EditorUiHostRuntimeError> {
        apply_template_control_attributes_to_surface(
            &body.document_id,
            surface,
            &template_v2_component_patch_attributes(body),
        )
    }

    pub(crate) fn bind_template_actions_for_pane(
        &self,
        pane_id: &str,
        surface: &mut UiSurface,
        host_model: &mut RetainedUiHostModel,
    ) -> Result<(), EditorUiHostRuntimeError> {
        let document_id = host_model.document_id.clone();
        let plugin_owner = self.plugin_v2_document_owner(&document_id);
        let control_attributes = template_control_attributes_from_host_model(host_model)?;
        let mut registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        let control_attributes = registry.rebind_pane(
            pane_id,
            &document_id,
            plugin_owner.as_ref(),
            control_attributes,
        );
        apply_template_control_attributes_to_host_model(
            &document_id,
            host_model,
            &control_attributes,
        )?;
        for node in &mut host_model.nodes {
            for binding in &mut node.bindings {
                let Some(action_source) = binding.template_action_source.clone() else {
                    continue;
                };
                binding.action_id = registry.bind_for_control(
                    pane_id,
                    &document_id,
                    &binding.binding_id,
                    plugin_owner.clone(),
                    node.control_id.as_deref(),
                    node.attributes.clone(),
                    action_source,
                    BTreeMap::new(),
                );
            }
        }
        drop(registry);
        apply_template_control_attributes_to_surface(&document_id, surface, &control_attributes)
    }

    pub(crate) fn update_template_action_control_state(
        &self,
        pane_id: &str,
        control_id: &str,
        attributes: &BTreeMap<String, toml::Value>,
    ) -> bool {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .update_control_attributes_for_pane(pane_id, control_id, attributes)
    }

    pub(crate) fn select_template_table_row(
        &self,
        pane_id: &str,
        control_id: &str,
        source_index: i32,
        identity_kind: &str,
        identity_text: &str,
    ) -> bool {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .select_table_row(
                pane_id,
                control_id,
                source_index,
                identity_kind,
                identity_text,
            )
    }

    pub(crate) fn remove_template_actions_for_pane(&self, pane_id: &str) {
        self.template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned")
            .remove_pane(pane_id);
    }

    pub(crate) fn dispatch_template_action_for_token<T>(
        &self,
        token: &str,
        dispatch: impl FnOnce(&UiTemplateActionInvocation) -> T,
    ) -> Option<T> {
        // Keep the active document owner and action slot stable through dispatch.
        let plugin_v2_documents = self
            .plugin_v2_documents
            .lock()
            .expect("plugin V2 document catalog mutex should not be poisoned");
        let registry = self
            .template_action_registry
            .lock()
            .expect("template action registry mutex should not be poisoned");
        registry
            .action_for_token(token, |document_id| {
                plugin_v2_documents
                    .get(document_id)
                    .map(|document| document.owner().clone())
            })
            .map(|action| dispatch(&action))
    }

    pub(crate) fn is_template_action_token(&self, token: &str) -> bool {
        token.starts_with("template-v2/")
    }
}

fn template_control_attributes_from_host_model(
    host_model: &RetainedUiHostModel,
) -> Result<BTreeMap<String, BTreeMap<String, toml::Value>>, EditorUiHostRuntimeError> {
    let mut controls = BTreeMap::new();
    for node in &host_model.nodes {
        let Some(control_id) = node.control_id.as_ref() else {
            continue;
        };
        if controls
            .insert(control_id.clone(), node.attributes.clone())
            .is_some()
        {
            return Err(EditorUiHostRuntimeError::DuplicateRetainedControl {
                document_id: host_model.document_id.clone(),
                control_id: control_id.clone(),
            });
        }
    }
    Ok(controls)
}

pub(super) fn apply_template_control_attributes_to_host_model(
    document_id: &str,
    host_model: &mut RetainedUiHostModel,
    control_attributes: &BTreeMap<String, BTreeMap<String, toml::Value>>,
) -> Result<(), EditorUiHostRuntimeError> {
    let mut control_node_indices = BTreeMap::<String, Vec<usize>>::new();
    for (index, node) in host_model.nodes.iter().enumerate() {
        let Some(control_id) = node.control_id.as_deref() else {
            continue;
        };
        control_node_indices
            .entry(control_id.to_string())
            .or_default()
            .push(index);
    }

    for (control_id, attributes) in control_attributes {
        let matching_node_indices = control_node_indices
            .get(control_id)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let [node_index] = matching_node_indices else {
            return Err(if matching_node_indices.is_empty() {
                EditorUiHostRuntimeError::MissingRetainedControl {
                    document_id: document_id.to_string(),
                    control_id: control_id.clone(),
                }
            } else {
                EditorUiHostRuntimeError::DuplicateRetainedControl {
                    document_id: document_id.to_string(),
                    control_id: control_id.clone(),
                }
            });
        };
        host_model.nodes[*node_index]
            .attributes
            .extend(attributes.clone());
    }
    Ok(())
}

pub(super) fn apply_template_control_attributes_to_surface(
    document_id: &str,
    surface: &mut UiSurface,
    control_attributes: &BTreeMap<String, BTreeMap<String, toml::Value>>,
) -> Result<(), EditorUiHostRuntimeError> {
    let mut control_node_ids = BTreeMap::<String, Vec<UiNodeId>>::new();
    for (node_id, node) in &surface.tree.nodes {
        let Some(control_id) = node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
        else {
            continue;
        };
        control_node_ids
            .entry(control_id.to_string())
            .or_default()
            .push(*node_id);
    }

    for (control_id, attributes) in control_attributes {
        let matching_node_ids = control_node_ids
            .get(control_id)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let [node_id] = matching_node_ids else {
            return Err(if matching_node_ids.is_empty() {
                EditorUiHostRuntimeError::MissingTemplateSurfaceControl {
                    document_id: document_id.to_string(),
                    control_id: control_id.clone(),
                }
            } else {
                EditorUiHostRuntimeError::DuplicateTemplateSurfaceControl {
                    document_id: document_id.to_string(),
                    control_id: control_id.clone(),
                }
            });
        };
        let disabled = attributes.get("disabled") == Some(&toml::Value::Boolean(true));
        for (property, value) in attributes {
            // `disabled` is terminal input state and must win over a contradictory `enabled` patch.
            if disabled && property == "enabled" {
                continue;
            }
            apply_template_control_property(
                document_id,
                control_id,
                surface,
                *node_id,
                property,
                UiValue::from_toml(value),
            )?;
        }
    }
    Ok(())
}

fn apply_template_control_property(
    document_id: &str,
    control_id: &str,
    surface: &mut UiSurface,
    node_id: UiNodeId,
    property: &str,
    value: UiValue,
) -> Result<(), EditorUiHostRuntimeError> {
    let report = surface.mutate_property(UiPropertyMutationRequest::new(
        node_id,
        property,
        value.clone(),
    ))?;
    if let Some(detail) = report.message {
        return Err(EditorUiHostRuntimeError::TemplateControlStateRejected {
            document_id: document_id.to_string(),
            control_id: control_id.to_string(),
            property: property.to_string(),
            detail,
        });
    }
    if property == "disabled" {
        let UiValue::Bool(disabled) = value else {
            return Err(EditorUiHostRuntimeError::TemplateControlStateRejected {
                document_id: document_id.to_string(),
                control_id: control_id.to_string(),
                property: property.to_string(),
                detail: "disabled expects a boolean value".to_string(),
            });
        };
        let report = surface.mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "enabled",
            UiValue::Bool(!disabled),
        ))?;
        if let Some(detail) = report.message {
            return Err(EditorUiHostRuntimeError::TemplateControlStateRejected {
                document_id: document_id.to_string(),
                control_id: control_id.to_string(),
                property: "enabled".to_string(),
                detail,
            });
        }
    }
    Ok(())
}
