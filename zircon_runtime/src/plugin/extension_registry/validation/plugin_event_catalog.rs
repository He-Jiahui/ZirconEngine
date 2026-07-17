use std::collections::BTreeSet;

use crate::plugin::{PluginEventCatalogManifest, RuntimeExtensionRegistryError};

pub(in crate::plugin::extension_registry) fn validate_plugin_event_catalog_manifest(
    descriptor: &PluginEventCatalogManifest,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_event_field("namespace", &descriptor.namespace)?;
    validate_dot_namespaced_event_id("event catalog", "namespace", &descriptor.namespace)?;
    if descriptor.version == 0 {
        return invalid_plugin_event_catalog(format!(
            "{} version must be a positive u32",
            descriptor.namespace
        ));
    }
    if descriptor.events.is_empty() {
        return invalid_plugin_event_catalog(format!(
            "{} must declare at least one event",
            descriptor.namespace
        ));
    }

    let package_namespace = descriptor.namespace.split('.').next().unwrap_or_default();
    let event_prefix = format!("{}.", descriptor.namespace);
    let payload_prefix = format!("{package_namespace}.");
    let mut event_ids = BTreeSet::new();
    for event in &descriptor.events {
        validate_plugin_event_field("id", &event.id)?;
        validate_dot_namespaced_event_id("event", "id", &event.id)?;
        if !event.id.starts_with(&event_prefix) {
            return invalid_plugin_event_catalog(format!(
                "event id `{}` must stay under catalog namespace `{}`",
                event.id, descriptor.namespace
            ));
        }
        if !event_ids.insert(event.id.as_str()) {
            return invalid_plugin_event_catalog(format!(
                "event id `{}` must be unique inside catalog `{}`",
                event.id, descriptor.namespace
            ));
        }
        validate_plugin_event_field("display_name", &event.display_name)?;

        if !event.payload_schema.is_empty() {
            validate_plugin_event_field("payload_schema", &event.payload_schema)?;
            validate_dot_namespaced_event_id("event", "payload_schema", &event.payload_schema)?;
            if !event.payload_schema.starts_with(&payload_prefix) {
                return invalid_plugin_event_catalog(format!(
                    "payload_schema `{}` must stay under package namespace `{payload_prefix}`",
                    event.payload_schema
                ));
            }
            validate_versioned_payload_schema(&event.payload_schema)?;
        }
    }

    Ok(())
}

fn validate_dot_namespaced_event_id(
    context: &str,
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if !value.contains('.') {
        return invalid_plugin_event_catalog(format!(
            "{context} {field_name} `{value}` must use at least two dot-separated namespace segments"
        ));
    }
    for segment in value.split('.') {
        if segment.is_empty()
            || !segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return invalid_plugin_event_catalog(format!(
                "{context} {field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots"
            ));
        }
    }
    Ok(())
}

fn validate_versioned_payload_schema(
    payload_schema: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    let version_segment = payload_schema.rsplit('.').next().unwrap_or(payload_schema);
    let Some(version_number) = version_segment.strip_prefix('v') else {
        return invalid_plugin_event_catalog(format!(
            "payload_schema `{payload_schema}` must end with a version segment like `v1`"
        ));
    };
    if version_number.is_empty()
        || !version_number.bytes().all(|byte| byte.is_ascii_digit())
        || version_number.starts_with('0')
    {
        return invalid_plugin_event_catalog(format!(
            "payload_schema `{payload_schema}` version segment must be a positive integer without leading zeroes"
        ));
    }
    Ok(())
}

fn validate_plugin_event_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return invalid_plugin_event_catalog(format!(
            "{field_name} `{value}` must be non-empty and trimmed"
        ));
    }
    Ok(())
}

fn invalid_plugin_event_catalog<T>(message: String) -> Result<T, RuntimeExtensionRegistryError> {
    Err(RuntimeExtensionRegistryError::InvalidPluginEventCatalog(
        message,
    ))
}
