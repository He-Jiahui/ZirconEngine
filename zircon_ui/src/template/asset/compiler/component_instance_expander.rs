use std::collections::BTreeMap;

use toml::Value;

use crate::{
    UiAssetDocument, UiAssetError, UiAssetKind, UiChildMount, UiComponentDefinition,
    UiNodeDefinition, UiTemplateNode,
};

use super::ui_document_compiler::{CompilationArtifacts, UiDocumentCompiler};
use super::value_normalizer::{
    append_classes, merge_value_maps, merge_value_maps_resolved, normalize_layout,
    resolve_value, resolve_value_map,
};

impl UiDocumentCompiler {
    pub(super) fn expand_component_instance(
        &self,
        document: &UiAssetDocument,
        component_name: &str,
        instance_node: &UiNodeDefinition,
        tokens: &BTreeMap<String, Value>,
        caller_document: &UiAssetDocument,
        caller_tokens: &BTreeMap<String, Value>,
        params: &BTreeMap<String, Value>,
        artifacts: &mut CompilationArtifacts,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let component = document.components.get(component_name).ok_or_else(|| {
            UiAssetError::UnknownComponent {
                asset_id: document.asset.id.clone(),
                component: component_name.to_string(),
            }
        })?;
        if document.asset.kind == UiAssetKind::Widget {
            artifacts.record_widget_styles(document, tokens);
        }

        validate_slot_mounts(component_name, component, &instance_node.children)?;

        let component_params =
            resolve_component_params(component, &instance_node.params, tokens, params);

        let mut fills = BTreeMap::new();
        for child in &instance_node.children {
            let mount_name = child.mount.clone().unwrap_or_default();
            let expanded = self.expand_node(
                caller_document,
                &child.child,
                caller_tokens,
                params,
                None,
                artifacts,
            )?;
            fills
                .entry(mount_name)
                .or_insert_with(Vec::new)
                .extend(apply_child_mount(expanded, child, tokens, params));
        }

        let component_tokens = super::value_normalizer::compose_tokens(tokens, &document.tokens);
        let mut roots = self.expand_node(
            document,
            &component.root,
            &component_tokens,
            &component_params,
            Some(&fills),
            artifacts,
        )?;
        if roots.len() != 1 {
            return Err(UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: format!("component {component_name} must expand to exactly one root node"),
            });
        }

        let mut root = roots.remove(0);
        decorate_component_root(&mut root, instance_node, tokens, params);
        Ok(vec![root])
    }
}

pub(super) fn apply_child_mount(
    nodes: Vec<UiTemplateNode>,
    child: &UiChildMount,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> Vec<UiTemplateNode> {
    let mut slot = resolve_value_map(&child.slot, tokens, params);
    normalize_layout(&mut slot);
    nodes
        .into_iter()
        .map(|mut node| {
            merge_value_maps(&mut node.slot_attributes, &slot);
            node
        })
        .collect()
}

fn resolve_component_params(
    component: &UiComponentDefinition,
    provided: &BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut resolved = BTreeMap::new();
    for (name, schema) in &component.params {
        if let Some(default) = &schema.default {
            let _ = resolved.insert(name.clone(), resolve_value(default, tokens, params));
        }
    }
    for (name, value) in provided {
        let _ = resolved.insert(name.clone(), resolve_value(value, tokens, params));
    }
    resolved
}

fn decorate_component_root(
    root: &mut UiTemplateNode,
    instance_node: &UiNodeDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) {
    if let Some(control_id) = &instance_node.control_id {
        root.control_id = Some(control_id.clone());
    }
    append_classes(&mut root.classes, &instance_node.classes);
    let inline = resolve_value_map(&instance_node.style_overrides.self_values, tokens, params);
    merge_value_maps(&mut root.style_overrides, &inline);
    merge_value_maps_resolved(
        &mut root.slot_attributes,
        &instance_node.style_overrides.slot,
        tokens,
        params,
    );
}

fn validate_slot_mounts(
    component_name: &str,
    component: &UiComponentDefinition,
    children: &[UiChildMount],
) -> Result<(), UiAssetError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for child in children {
        let slot_name = child.mount.clone().unwrap_or_default();
        let slot = component
            .slots
            .get(&slot_name)
            .ok_or_else(|| UiAssetError::UnknownSlot {
                component: component_name.to_string(),
                slot_name: slot_name.clone(),
            })?;
        let count = counts.entry(slot_name.clone()).or_insert(0);
        *count += 1;
        if !slot.multiple && *count > 1 {
            return Err(UiAssetError::SlotDoesNotAcceptMultiple {
                component: component_name.to_string(),
                slot_name,
            });
        }
    }

    for (slot_name, slot) in &component.slots {
        if slot.required && !counts.contains_key(slot_name) {
            return Err(UiAssetError::MissingRequiredSlot {
                component: component_name.to_string(),
                slot_name: slot_name.clone(),
            });
        }
    }

    Ok(())
}

pub(super) fn component_name_from_reference(reference: &str) -> Result<String, UiAssetError> {
    reference
        .split_once('#')
        .map(|(_, component)| component.to_string())
        .ok_or_else(|| UiAssetError::InvalidDocument {
            asset_id: reference.to_string(),
            detail: "component references must include a #Component suffix".to_string(),
        })
}
