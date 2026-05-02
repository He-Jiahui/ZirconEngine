use std::collections::BTreeMap;

use serde::Deserialize;
use toml::Value;

use zircon_runtime_interface::ui::template::UiBindingRef;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiChildMount,
    UiComponentDefinition, UiComponentParamSchema, UiNamedSlotSchema, UiNodeDefinition,
    UiNodeDefinitionKind, UiStyleDeclarationBlock, UiStyleScope, UiStyleSheet,
};

pub(super) fn migrate_flat_toml_str(input: &str) -> Result<UiAssetDocument, UiAssetError> {
    let legacy: FlatUiAssetDocument =
        toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    legacy.into_tree_document()
}

impl FlatUiAssetDocument {
    fn into_tree_document(self) -> Result<UiAssetDocument, UiAssetError> {
        let root = self
            .root
            .map(|root| build_tree_node(&self.asset.id, &self.nodes, &root.node, &mut Vec::new()))
            .transpose()?;
        let components = self
            .components
            .into_iter()
            .map(|(name, component)| {
                Ok((
                    name,
                    UiComponentDefinition {
                        root: build_tree_node(
                            &self.asset.id,
                            &self.nodes,
                            &component.root,
                            &mut Vec::new(),
                        )?,
                        style_scope: component.style_scope,
                        contract: Default::default(),
                        params: component.params,
                        slots: component.slots,
                    },
                ))
            })
            .collect::<Result<_, UiAssetError>>()?;

        Ok(UiAssetDocument {
            asset: self.asset,
            imports: self.imports,
            tokens: self.tokens,
            root,
            components,
            stylesheets: self.stylesheets,
        })
    }
}

fn build_tree_node(
    asset_id: &str,
    nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    node_id: &str,
    visiting: &mut Vec<String>,
) -> Result<UiNodeDefinition, UiAssetError> {
    if visiting.iter().any(|current| current == node_id) {
        return Err(UiAssetError::InvalidDocument {
            asset_id: asset_id.to_string(),
            detail: format!("ui asset tree contains a cycle at {node_id}"),
        });
    }
    let node = nodes
        .get(node_id)
        .ok_or_else(|| UiAssetError::MissingNode {
            asset_id: asset_id.to_string(),
            node_id: node_id.to_string(),
        })?;

    visiting.push(node_id.to_string());
    let children = node
        .children
        .iter()
        .map(|child| {
            Ok(UiChildMount {
                mount: child.mount.clone(),
                slot: child.slot.clone(),
                node: build_tree_node(asset_id, nodes, &child.child, visiting)?,
            })
        })
        .collect::<Result<Vec<_>, UiAssetError>>()?;
    let _ = visiting.pop();

    Ok(UiNodeDefinition {
        node_id: node_id.to_string(),
        kind: node.kind,
        widget_type: node.widget_type.clone(),
        component: node.component.clone(),
        component_ref: node.component_ref.clone(),
        component_api_version: None,
        slot_name: node.slot_name.clone(),
        control_id: node.control_id.clone(),
        classes: node.classes.clone(),
        params: node.params.clone(),
        props: node.props.clone(),
        layout: node.layout.clone(),
        bindings: node.bindings.clone(),
        style_overrides: node.style_overrides.clone(),
        children,
    })
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct FlatUiAssetDocument {
    pub asset: UiAssetHeader,
    #[serde(default)]
    pub imports: UiAssetImports,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    #[serde(default)]
    pub root: Option<FlatUiAssetRoot>,
    #[serde(default)]
    pub nodes: BTreeMap<String, FlatUiNodeDefinition>,
    #[serde(default)]
    pub components: BTreeMap<String, FlatUiComponentDefinition>,
    #[serde(default)]
    pub stylesheets: Vec<UiStyleSheet>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct FlatUiAssetRoot {
    pub node: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct FlatUiNodeDefinition {
    #[serde(default)]
    pub kind: UiNodeDefinitionKind,
    #[serde(default, rename = "type")]
    pub widget_type: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub component_ref: Option<String>,
    #[serde(default)]
    pub slot_name: Option<String>,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub bindings: Vec<UiBindingRef>,
    #[serde(default)]
    pub style_overrides: UiStyleDeclarationBlock,
    #[serde(default)]
    pub children: Vec<FlatUiChildMount>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct FlatUiChildMount {
    pub child: String,
    #[serde(default)]
    pub mount: Option<String>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct FlatUiComponentDefinition {
    pub root: String,
    #[serde(default)]
    pub style_scope: UiStyleScope,
    #[serde(default)]
    pub params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    pub slots: BTreeMap<String, UiNamedSlotSchema>,
}
