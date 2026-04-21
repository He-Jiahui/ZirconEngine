use std::collections::BTreeMap;

#[cfg(test)]
use serde::Deserialize;
use toml::Value;

#[cfg(test)]
use crate::ui::template::UiComponentParamSchema;
use crate::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiAssetKind, UiChildMount,
    UiComponentDefinition, UiNamedSlotSchema, UiNodeDefinition, UiNodeDefinitionKind,
    UiStyleDeclarationBlock, UiStyleScope,
};
use crate::ui::template::{UiTemplateDocument, UiTemplateNode};

#[derive(Default)]
pub struct UiLegacyTemplateAdapter;

#[cfg(test)]
#[derive(Default)]
pub struct UiFlatAssetMigrationAdapter;

impl UiLegacyTemplateAdapter {
    pub fn layout_document(
        asset_id: impl Into<String>,
        display_name: impl Into<String>,
        document: &UiTemplateDocument,
    ) -> Result<UiAssetDocument, UiAssetError> {
        let mut components = BTreeMap::new();

        for (name, template) in &document.components {
            let component_root = format!("component_{name}_root");
            let _ = components.insert(
                name.clone(),
                UiComponentDefinition {
                    root: convert_template_node(&component_root, &template.root)?,
                    style_scope: UiStyleScope::Closed,
                    params: BTreeMap::new(),
                    slots: template
                        .slots
                        .iter()
                        .map(|(slot_name, slot)| {
                            (
                                slot_name.clone(),
                                UiNamedSlotSchema {
                                    required: slot.required,
                                    multiple: slot.multiple,
                                },
                            )
                        })
                        .collect(),
                },
            );
        }

        Ok(UiAssetDocument {
            asset: UiAssetHeader {
                kind: UiAssetKind::Layout,
                id: asset_id.into(),
                version: document.version,
                display_name: display_name.into(),
            },
            imports: UiAssetImports::default(),
            tokens: BTreeMap::new(),
            root: Some(convert_template_node("root", &document.root)?),
            components,
            stylesheets: Vec::new(),
        })
    }

    pub fn layout_source(
        asset_id: impl Into<String>,
        display_name: impl Into<String>,
        document: &UiTemplateDocument,
    ) -> Result<String, UiAssetError> {
        let document = Self::layout_document(asset_id, display_name, document)?;
        toml::to_string_pretty(&document)
            .map_err(|error| UiAssetError::ParseToml(error.to_string()))
    }
}

#[cfg(test)]
impl UiFlatAssetMigrationAdapter {
    pub fn migrate_toml_str(input: &str) -> Result<String, UiAssetError> {
        let legacy: FlatUiAssetDocument =
            toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
        let migrated = legacy.into_tree_document()?;
        toml::to_string_pretty(&migrated)
            .map_err(|error| UiAssetError::ParseToml(error.to_string()))
    }
}

fn convert_template_node(
    node_id: &str,
    node: &UiTemplateNode,
) -> Result<UiNodeDefinition, UiAssetError> {
    let (kind, widget_type, component, slot_name) = if let Some(component_name) = &node.component {
        (
            UiNodeDefinitionKind::Native,
            Some(component_name.clone()),
            None,
            None,
        )
    } else if let Some(template_name) = &node.template {
        (
            UiNodeDefinitionKind::Component,
            None,
            Some(template_name.clone()),
            None,
        )
    } else if let Some(slot_name) = &node.slot {
        (
            UiNodeDefinitionKind::Slot,
            None,
            None,
            Some(slot_name.clone()),
        )
    } else {
        return Err(UiAssetError::InvalidDocument {
            asset_id: "legacy-template".to_string(),
            detail: format!("legacy node {node_id} missing component/template/slot"),
        });
    };

    let mut props = BTreeMap::new();
    let mut layout = None;
    for (key, value) in &node.attributes {
        if key == "layout" {
            layout = value.as_table().map(table_to_btree_map);
        } else {
            let _ = props.insert(key.clone(), value.clone());
        }
    }

    let mut children = Vec::new();
    for (index, child) in node.children.iter().enumerate() {
        let child_id = format!("{node_id}_{index}");
        children.push(UiChildMount {
            mount: None,
            slot: child.slot_attributes.clone(),
            node: convert_template_node(&child_id, child)?,
        });
    }

    for (slot_name, filled) in &node.slots {
        for (index, child) in filled.iter().enumerate() {
            let child_id = format!("{node_id}_{slot_name}_{index}");
            children.push(UiChildMount {
                mount: Some(slot_name.clone()),
                slot: child.slot_attributes.clone(),
                node: convert_template_node(&child_id, child)?,
            });
        }
    }

    Ok(UiNodeDefinition {
        node_id: node_id.to_string(),
        kind,
        widget_type,
        component,
        component_ref: None,
        slot_name,
        control_id: node.control_id.clone(),
        classes: node.classes.clone(),
        params: BTreeMap::new(),
        props,
        layout,
        bindings: node.bindings.clone(),
        style_overrides: UiStyleDeclarationBlock {
            self_values: node.style_overrides.clone(),
            slot: node.slot_attributes.clone(),
        },
        children,
    })
}

fn table_to_btree_map(table: &toml::map::Map<String, Value>) -> BTreeMap<String, Value> {
    table
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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
    pub stylesheets: Vec<crate::ui::template::UiStyleSheet>,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Deserialize)]
struct FlatUiAssetRoot {
    pub node: String,
}

#[cfg(test)]
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
    pub bindings: Vec<crate::ui::template::UiBindingRef>,
    #[serde(default)]
    pub style_overrides: UiStyleDeclarationBlock,
    #[serde(default)]
    pub children: Vec<FlatUiChildMount>,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct FlatUiChildMount {
    pub child: String,
    #[serde(default)]
    pub mount: Option<String>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
}

#[cfg(test)]
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
