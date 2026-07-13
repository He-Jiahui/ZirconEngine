use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::{RuntimeExtensionRegistryError, UiComponentDescriptor};
use std::collections::BTreeSet;

use super::is_lowercase_plugin_package_id;

pub(in crate::plugin::extension_registry) fn validate_component_type_descriptor(
    descriptor: &ComponentTypeDescriptor,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_component_type_field("type_id", &descriptor.type_id)?;
    validate_component_type_field("plugin_id", &descriptor.plugin_id)?;
    validate_component_type_plugin_id(&descriptor.plugin_id)?;
    validate_component_type_field("display_name", &descriptor.display_name)?;
    let expected_prefix = format!("{}.", descriptor.plugin_id);
    if !descriptor.type_id.starts_with(&expected_prefix) {
        return Err(RuntimeExtensionRegistryError::InvalidComponentType(
            format!(
                "component type {} must be prefixed by plugin id {}",
                descriptor.type_id, descriptor.plugin_id
            ),
        ));
    }

    let mut property_names = BTreeSet::new();
    for property in &descriptor.properties {
        validate_component_type_field("property name", &property.name)?;
        validate_component_type_field("property value_type", &property.value_type)?;
        if !property_names.insert(property.name.as_str()) {
            return Err(RuntimeExtensionRegistryError::InvalidComponentType(
                format!(
                    "component type {} property `{}` must be unique",
                    descriptor.type_id, property.name
                ),
            ));
        }
    }
    Ok(())
}

pub(in crate::plugin::extension_registry) fn validate_ui_component_descriptor(
    descriptor: &UiComponentDescriptor,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_ui_component_field("component_id", &descriptor.component_id)?;
    validate_ui_component_field("plugin_id", &descriptor.plugin_id)?;
    validate_ui_component_plugin_id(&descriptor.plugin_id)?;
    validate_ui_component_field("ui_document", &descriptor.ui_document)?;
    let expected_prefix = format!("{}.", descriptor.plugin_id);
    if !descriptor.component_id.starts_with(&expected_prefix) {
        return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
            "ui component {} must be prefixed by plugin id {}",
            descriptor.component_id, descriptor.plugin_id
        )));
    }
    if !descriptor.ui_document.ends_with(".zui") {
        return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
            "ui component {} document `{}` must reference a .zui component asset",
            descriptor.component_id, descriptor.ui_document
        )));
    }
    Ok(())
}

fn validate_component_type_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(RuntimeExtensionRegistryError::InvalidComponentType(
            format!("{field_name} `{value}` must be non-empty and trimmed"),
        ));
    }
    Ok(())
}

fn validate_component_type_plugin_id(plugin_id: &str) -> Result<(), RuntimeExtensionRegistryError> {
    if !is_lowercase_plugin_package_id(plugin_id) {
        return Err(RuntimeExtensionRegistryError::InvalidComponentType(
            format!(
                "plugin_id `{plugin_id}` must start with a lowercase ASCII letter and contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments without trailing or repeated underscores"
            ),
        ));
    }
    Ok(())
}

fn validate_ui_component_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
            "{field_name} `{value}` must be non-empty and trimmed"
        )));
    }
    Ok(())
}

fn validate_ui_component_plugin_id(plugin_id: &str) -> Result<(), RuntimeExtensionRegistryError> {
    if !is_lowercase_plugin_package_id(plugin_id) {
        return Err(RuntimeExtensionRegistryError::InvalidUiComponent(format!(
            "plugin_id `{plugin_id}` must start with a lowercase ASCII letter and contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments without trailing or repeated underscores"
        )));
    }
    Ok(())
}
