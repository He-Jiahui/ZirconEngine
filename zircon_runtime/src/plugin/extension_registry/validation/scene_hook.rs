use crate::plugin::{RuntimeExtensionRegistryError, SceneRuntimeHookRegistration};

use super::is_lowercase_plugin_package_id;

pub(in crate::plugin::extension_registry) fn validate_scene_hook_registration(
    registration: &SceneRuntimeHookRegistration,
) -> Result<(), RuntimeExtensionRegistryError> {
    let descriptor = registration.descriptor();
    validate_scene_hook_field("id", &descriptor.id)?;
    validate_scene_hook_namespace("id", &descriptor.id)?;
    validate_scene_hook_field("plugin_id", &descriptor.plugin_id)?;
    validate_scene_hook_plugin_id(&descriptor.plugin_id)?;
    let expected_prefix = format!("{}.", descriptor.plugin_id);
    if !descriptor.id.starts_with(&expected_prefix) {
        return Err(RuntimeExtensionRegistryError::InvalidSceneHook(format!(
            "scene hook {} must be prefixed by plugin id {}",
            descriptor.id, descriptor.plugin_id
        )));
    }
    Ok(())
}

fn validate_scene_hook_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(RuntimeExtensionRegistryError::InvalidSceneHook(format!(
            "{field_name} `{value}` must be non-empty and trimmed"
        )));
    }
    Ok(())
}

fn validate_scene_hook_namespace(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    let segments: Vec<_> = value.split('.').collect();
    if segments.len() < 2 {
        return Err(RuntimeExtensionRegistryError::InvalidSceneHook(format!(
            "{field_name} `{value}` must use at least two dot-separated namespace segments"
        )));
    }
    for segment in segments {
        validate_scene_hook_segment(field_name, value, segment)?;
    }
    Ok(())
}

fn validate_scene_hook_plugin_id(plugin_id: &str) -> Result<(), RuntimeExtensionRegistryError> {
    if !is_lowercase_plugin_package_id(plugin_id) {
        return Err(RuntimeExtensionRegistryError::InvalidSceneHook(format!(
            "plugin_id `{plugin_id}` must start with a lowercase ASCII letter and contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments without trailing or repeated underscores"
        )));
    }
    Ok(())
}

fn validate_scene_hook_segment(
    field_name: &str,
    value: &str,
    segment: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if segment.is_empty()
        || !segment.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
    {
        return Err(RuntimeExtensionRegistryError::InvalidSceneHook(format!(
            "{field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, hyphens, and dots"
        )));
    }
    Ok(())
}
