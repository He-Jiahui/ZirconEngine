use std::collections::BTreeMap;

use toml::Value;

use crate::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetKind, UiAssetRoot, UiChildMount,
    UiComponentDefinition, UiNamedSlotSchema, UiNodeDefinition, UiNodeDefinitionKind,
    UiStyleDeclarationBlock,
};
use crate::ui::template::{UiTemplateDocument, UiTemplateNode};

#[derive(Default)]
pub struct UiLegacyTemplateAdapter;

impl UiLegacyTemplateAdapter {
    pub fn layout_document(
        asset_id: impl Into<String>,
        display_name: impl Into<String>,
        document: &UiTemplateDocument,
    ) -> Result<UiAssetDocument, UiAssetError> {
        let mut nodes = BTreeMap::new();
        let mut components = BTreeMap::new();

        convert_template_node("root", &document.root, &mut nodes)?;
        for (name, template) in &document.components {
            let component_root = format!("component_{name}_root");
            convert_template_node(&component_root, &template.root, &mut nodes)?;
            let _ = components.insert(
                name.clone(),
                UiComponentDefinition {
                    root: component_root,
                    style_scope: Default::default(),
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
            imports: Default::default(),
            tokens: BTreeMap::new(),
            root: Some(UiAssetRoot {
                node: "root".to_string(),
            }),
            nodes,
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

fn convert_template_node(
    node_id: &str,
    node: &UiTemplateNode,
    nodes: &mut BTreeMap<String, UiNodeDefinition>,
) -> Result<(), UiAssetError> {
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
        convert_template_node(&child_id, child, nodes)?;
        children.push(UiChildMount {
            child: child_id,
            mount: None,
            slot: child.slot_attributes.clone(),
        });
    }

    for (slot_name, filled) in &node.slots {
        for (index, child) in filled.iter().enumerate() {
            let child_id = format!("{node_id}_{slot_name}_{index}");
            convert_template_node(&child_id, child, nodes)?;
            children.push(UiChildMount {
                child: child_id,
                mount: Some(slot_name.clone()),
                slot: child.slot_attributes.clone(),
            });
        }
    }

    let _ = nodes.insert(
        node_id.to_string(),
        UiNodeDefinition {
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
        },
    );

    Ok(())
}

fn table_to_btree_map(table: &toml::map::Map<String, Value>) -> BTreeMap<String, Value> {
    table
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}
