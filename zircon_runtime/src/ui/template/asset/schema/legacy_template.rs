use std::collections::BTreeMap;

use toml::Value;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiAssetKind, UiChildMount,
    UiComponentDefinition, UiNamedSlotSchema, UiNodeDefinition, UiNodeDefinitionKind,
    UiStyleDeclarationBlock, UiStyleScope,
};
use zircon_runtime_interface::ui::template::{UiTemplateDocument, UiTemplateNode};

pub(super) fn convert_legacy_template_document(
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
                contract: Default::default(),
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
            detail: format!("template fixture node {node_id} missing component/template/slot"),
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
        component_api_version: None,
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
