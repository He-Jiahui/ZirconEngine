use std::collections::BTreeMap;

use toml::Value;

use crate::ui::template::{
    UiAssetDocument, UiAssetError, UiNodeDefinition, UiNodeDefinitionKind, UiTemplateNode,
};

use super::component_instance_expander::{apply_child_mount, component_name_from_reference};
use super::ui_document_compiler::{CompilationArtifacts, UiDocumentCompiler};
use super::value_normalizer::{build_attribute_map, compose_tokens};

impl UiDocumentCompiler {
    pub(super) fn expand_node(
        &self,
        document: &UiAssetDocument,
        node: &UiNodeDefinition,
        tokens: &BTreeMap<String, Value>,
        params: &BTreeMap<String, Value>,
        slot_fills: Option<&BTreeMap<String, Vec<UiTemplateNode>>>,
        artifacts: &mut CompilationArtifacts,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let node_id = node.node_id.as_str();

        match node.kind {
            UiNodeDefinitionKind::Native => {
                self.expand_native_node(document, node, tokens, params, slot_fills, artifacts)
            }
            UiNodeDefinitionKind::Component => {
                let component_name =
                    node.component
                        .as_deref()
                        .ok_or_else(|| UiAssetError::InvalidDocument {
                            asset_id: document.asset.id.clone(),
                            detail: format!("component node {node_id} missing component name"),
                        })?;
                self.expand_component_instance(
                    document,
                    component_name,
                    node,
                    tokens,
                    document,
                    tokens,
                    params,
                    artifacts,
                )
            }
            UiNodeDefinitionKind::Reference => {
                let reference =
                    node.component_ref
                        .as_deref()
                        .ok_or_else(|| UiAssetError::InvalidDocument {
                            asset_id: document.asset.id.clone(),
                            detail: format!("reference node {node_id} missing component_ref"),
                        })?;
                let imported = self.widget_imports.get(reference).ok_or_else(|| {
                    UiAssetError::UnknownImport {
                        reference: reference.to_string(),
                    }
                })?;
                let component_name = component_name_from_reference(reference)?;
                self.expand_component_instance(
                    imported,
                    &component_name,
                    node,
                    &compose_tokens(tokens, &imported.tokens),
                    document,
                    tokens,
                    params,
                    artifacts,
                )
            }
            UiNodeDefinitionKind::Slot => {
                let slot_name =
                    node.slot_name
                        .as_deref()
                        .ok_or_else(|| UiAssetError::InvalidDocument {
                            asset_id: document.asset.id.clone(),
                            detail: format!("slot node {node_id} missing slot_name"),
                        })?;
                Ok(slot_fills
                    .and_then(|fills| fills.get(slot_name))
                    .cloned()
                    .unwrap_or_default())
            }
        }
    }

    fn expand_native_node(
        &self,
        document: &UiAssetDocument,
        node: &UiNodeDefinition,
        tokens: &BTreeMap<String, Value>,
        params: &BTreeMap<String, Value>,
        slot_fills: Option<&BTreeMap<String, Vec<UiTemplateNode>>>,
        artifacts: &mut CompilationArtifacts,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let component = node
            .widget_type
            .as_ref()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: "native node missing type".to_string(),
            })?
            .clone();
        let mut children = Vec::new();
        for child in &node.children {
            let expanded =
                self.expand_node(document, &child.node, tokens, params, slot_fills, artifacts)?;
            children.extend(apply_child_mount(expanded, child, tokens, params));
        }

        Ok(vec![UiTemplateNode {
            component: Some(component),
            template: None,
            slot: None,
            control_id: node.control_id.clone(),
            classes: node.classes.clone(),
            bindings: node.bindings.clone(),
            children,
            slots: BTreeMap::new(),
            attributes: build_attribute_map(node, tokens, params),
            slot_attributes: BTreeMap::new(),
            style_overrides: super::value_normalizer::resolve_value_map(
                &node.style_overrides.self_values,
                tokens,
                params,
            ),
            style_tokens: BTreeMap::new(),
        }])
    }
}
