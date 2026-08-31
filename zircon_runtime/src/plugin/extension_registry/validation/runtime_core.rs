use crate::core::ModuleDescriptor;
use crate::plugin::RuntimeExtensionRegistryError;

use super::is_lowercase_plugin_token;

pub(in crate::plugin::extension_registry) fn validate_manager_plugin_id(
    plugin_id: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if !is_lowercase_plugin_token(plugin_id) {
        return Err(RuntimeExtensionRegistryError::InvalidManager(format!(
            "plugin_id `{plugin_id}` must contain only lowercase ASCII letters, digits, and underscores"
        )));
    }
    Ok(())
}

pub(in crate::plugin::extension_registry) fn validate_module_descriptor(
    descriptor: &ModuleDescriptor,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_module_field("name", &descriptor.name)?;
    validate_module_field("description", &descriptor.description)?;
    Ok(())
}

fn validate_module_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if !module_field_is_valid(value) {
        return Err(RuntimeExtensionRegistryError::InvalidModule(format!(
            "{field_name} `{value}` must be non-empty and trimmed"
        )));
    }
    Ok(())
}

fn module_field_is_valid(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.len() == value.len()
}

#[cfg(test)]
#[path = "runtime_core/single_trim_tests.rs"]
mod single_trim_tests;
