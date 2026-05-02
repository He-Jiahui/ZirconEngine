use std::collections::BTreeMap;

use toml::Value;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiPropSchema, UiValue, UiValueKind,
};
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError, UiNodeDefinition};

use super::value_normalizer::{build_attribute_map, merge_value_maps};

pub(super) fn build_component_attribute_map(
    document: &UiAssetDocument,
    component_id: &str,
    node: &UiNodeDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
    descriptor: Option<&UiComponentDescriptor>,
) -> Result<BTreeMap<String, Value>, UiAssetError> {
    let authored = build_attribute_map(node, tokens, params);
    let Some(descriptor) = descriptor else {
        return Ok(authored);
    };

    let mut attributes = descriptor_default_attributes(descriptor);
    merge_value_maps(&mut attributes, &authored);
    validate_component_attributes(document, component_id, &attributes, descriptor)?;
    Ok(attributes)
}

fn descriptor_default_attributes(descriptor: &UiComponentDescriptor) -> BTreeMap<String, Value> {
    let mut attributes = BTreeMap::new();

    for (name, value) in &descriptor.default_props {
        let _ = attributes.insert(name.clone(), value.to_toml());
    }

    for schema in &descriptor.prop_schema {
        if let Some(value) = &schema.default_value {
            attributes
                .entry(schema.name.clone())
                .or_insert_with(|| value.to_toml());
        }
    }

    attributes
}

fn validate_component_attributes(
    document: &UiAssetDocument,
    component_id: &str,
    attributes: &BTreeMap<String, Value>,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiAssetError> {
    for schema in &descriptor.prop_schema {
        let Some(value) = attributes.get(&schema.name) else {
            if schema.required {
                return Err(UiAssetError::InvalidDocument {
                    asset_id: document.asset.id.clone(),
                    detail: format!(
                        "component {component_id} missing required prop {}",
                        schema.name
                    ),
                });
            }
            continue;
        };

        validate_component_prop(document, component_id, schema, value)?;
    }

    Ok(())
}

fn validate_component_prop(
    document: &UiAssetDocument,
    component_id: &str,
    schema: &UiPropSchema,
    value: &Value,
) -> Result<(), UiAssetError> {
    let Some(typed_value) = component_prop_value(value, schema.value_kind) else {
        return Err(UiAssetError::InvalidDocument {
            asset_id: document.asset.id.clone(),
            detail: format!(
                "component {component_id} prop {} expected {:?}",
                schema.name, schema.value_kind
            ),
        });
    };

    if let Some(number) = typed_value.as_f64() {
        if let Some(min) = schema.min {
            if number < min {
                return Err(UiAssetError::InvalidDocument {
                    asset_id: document.asset.id.clone(),
                    detail: format!(
                        "component {component_id} prop {} below minimum {min}",
                        schema.name
                    ),
                });
            }
        }
        if let Some(max) = schema.max {
            if number > max {
                return Err(UiAssetError::InvalidDocument {
                    asset_id: document.asset.id.clone(),
                    detail: format!(
                        "component {component_id} prop {} above maximum {max}",
                        schema.name
                    ),
                });
            }
        }
    }

    Ok(())
}

fn component_prop_value(value: &Value, kind: UiValueKind) -> Option<UiValue> {
    UiValue::from_toml_with_kind(value, kind).or_else(|| {
        (kind == UiValueKind::String && is_localized_text_ref(value))
            .then_some(UiValue::String(String::new()))
    })
}

fn is_localized_text_ref(value: &Value) -> bool {
    let Value::Table(table) = value else {
        return false;
    };
    table
        .get("text_key")
        .and_then(Value::as_str)
        .is_some_and(|key| !key.trim().is_empty())
}
