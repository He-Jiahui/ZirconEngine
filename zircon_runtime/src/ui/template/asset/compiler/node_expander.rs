use std::collections::BTreeMap;

use toml::Value;

use zircon_runtime_interface::ui::template::{
    parse_component_reference, UiAssetDocument, UiAssetError, UiNodeDefinition,
    UiNodeDefinitionKind, UiTemplateNode,
};

use super::component_instance_expander::apply_child_mount;
use super::component_props::build_component_attribute_map;
use super::control_scope::{
    resolve_binding_control_scope, resolve_optional_control_id, UiComponentControlScope,
};
use super::ui_document_compiler::{CompilationArtifacts, UiDocumentCompiler};
use super::value_normalizer::compose_tokens;

impl UiDocumentCompiler {
    pub(super) fn expand_node(
        &self,
        document: &UiAssetDocument,
        node: &UiNodeDefinition,
        tokens: &BTreeMap<String, Value>,
        params: &BTreeMap<String, Value>,
        slot_fills: Option<&BTreeMap<String, Vec<UiTemplateNode>>>,
        control_scope: Option<&UiComponentControlScope>,
        artifacts: &mut CompilationArtifacts,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let node_id = node.node_id.as_str();

        match node.kind {
            UiNodeDefinitionKind::Native => self.expand_native_node(
                document,
                node,
                tokens,
                params,
                slot_fills,
                control_scope,
                artifacts,
            ),
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
                    control_scope,
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
                let component_name = parse_component_reference(reference)
                    .map(|(_, component)| component.to_string())?;
                self.expand_component_instance(
                    imported,
                    &component_name,
                    node,
                    &compose_tokens(tokens, &imported.tokens),
                    document,
                    tokens,
                    params,
                    control_scope,
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
        control_scope: Option<&UiComponentControlScope>,
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
            let expanded = self.expand_node(
                document,
                &child.node,
                tokens,
                params,
                slot_fills,
                control_scope,
                artifacts,
            )?;
            children.extend(apply_child_mount(expanded, child, tokens, params));
        }

        let descriptor = self.component_descriptor(&component);
        let attributes =
            build_component_attribute_map(document, &component, node, tokens, params, descriptor)?;
        let widget = super::component_props::resolve_component_widget_contract(
            node.widget.clone().unwrap_or_default(),
            descriptor,
        );
        let bindings = resolve_binding_control_scope(
            node.bindings.clone(),
            control_scope,
            &document.asset.id,
        )?;
        let binding_source_asset_ids = vec![document.asset.id.clone(); bindings.len()];

        Ok(vec![UiTemplateNode {
            source_asset_id: Some(document.asset.id.clone()),
            component: Some(component),
            template: None,
            slot: None,
            control_id: resolve_optional_control_id(control_scope, node.control_id.as_deref()),
            classes: node.classes.clone(),
            bindings,
            binding_source_asset_ids,
            children,
            slots: BTreeMap::new(),
            attributes,
            slot_attributes: BTreeMap::new(),
            style_overrides: super::value_normalizer::resolve_value_map(
                &node.style_overrides.self_values,
                tokens,
                params,
            ),
            style_tokens: BTreeMap::new(),
            focus: node.focus.clone().unwrap_or_default(),
            navigation: node.navigation.clone().unwrap_or_default(),
            picking: node.picking.unwrap_or_default(),
            a11y: node.a11y.clone().unwrap_or_default(),
            widget,
            ..UiTemplateNode::default()
        }])
    }
}
