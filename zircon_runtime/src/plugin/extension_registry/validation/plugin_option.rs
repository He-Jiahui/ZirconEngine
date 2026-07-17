use std::collections::BTreeSet;

use crate::plugin::{PluginOptionManifest, RuntimeExtensionRegistryError};

pub(in crate::plugin::extension_registry) fn validate_plugin_option_manifest(
    descriptor: &PluginOptionManifest,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_option_field("key", &descriptor.key)?;
    validate_plugin_option_key(&descriptor.key)?;
    validate_plugin_option_field("display_name", &descriptor.display_name)?;
    validate_plugin_option_field("value_type", &descriptor.value_type)?;
    validate_plugin_option_field("default_value", &descriptor.default_value)?;
    if let Some(required_capability) = &descriptor.required_capability {
        validate_plugin_option_field("required_capability", required_capability)?;
        validate_plugin_option_capability(required_capability)?;
    }

    match descriptor.value_type.as_str() {
        "bool" => {
            if !matches!(descriptor.default_value.as_str(), "true" | "false") {
                return invalid_plugin_option(format!(
                    "{} bool default_value `{}` must be true or false",
                    descriptor.key, descriptor.default_value
                ));
            }
        }
        "integer" => {
            if descriptor.default_value.parse::<i64>().is_err() {
                return invalid_plugin_option(format!(
                    "{} integer default_value `{}` must parse as i64",
                    descriptor.key, descriptor.default_value
                ));
            }
        }
        "number" => match descriptor.default_value.parse::<f64>() {
            Ok(number) if number.is_finite() => {}
            _ => {
                return invalid_plugin_option(format!(
                    "{} number default_value `{}` must parse as a finite f64",
                    descriptor.key, descriptor.default_value
                ));
            }
        },
        "string" => {
            if !descriptor.enum_values.is_empty() {
                return invalid_plugin_option(format!(
                    "{} non-enum option must not declare enum_values",
                    descriptor.key
                ));
            }
        }
        "enum" => validate_plugin_option_enum_values(descriptor)?,
        _ => {
            return invalid_plugin_option(format!(
                "{} value_type `{}` must be bool, integer, number, string, or enum",
                descriptor.key, descriptor.value_type
            ));
        }
    }

    if descriptor.value_type != "enum" && !descriptor.enum_values.is_empty() {
        return invalid_plugin_option(format!(
            "{} non-enum option must not declare enum_values",
            descriptor.key
        ));
    }
    Ok(())
}

fn validate_plugin_option_key(option_key: &str) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_option_namespace("key", option_key)
}

fn validate_plugin_option_capability(
    required_capability: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_option_namespace("required_capability", required_capability)
}

fn validate_plugin_option_namespace(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if !value.contains('.') {
        return invalid_plugin_option(format!(
            "{field_name} `{value}` must use at least two dot-separated namespace segments"
        ));
    }
    for segment in value.split('.') {
        if segment.is_empty()
            || !segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return invalid_plugin_option(format!(
                "{field_name} `{value}` must contain only lowercase ASCII letters, digits, underscores, and dots"
            ));
        }
    }
    Ok(())
}

fn validate_plugin_option_enum_values(
    descriptor: &PluginOptionManifest,
) -> Result<(), RuntimeExtensionRegistryError> {
    if descriptor.enum_values.is_empty() {
        return invalid_plugin_option(format!(
            "{} enum option must declare enum_values",
            descriptor.key
        ));
    }

    validate_plugin_option_enum_token(&descriptor.key, "default_value", &descriptor.default_value)?;
    let mut seen_values = BTreeSet::new();
    for enum_value in &descriptor.enum_values {
        validate_plugin_option_enum_token(&descriptor.key, "enum_values", enum_value)?;
        if !seen_values.insert(enum_value) {
            return invalid_plugin_option(format!(
                "{} enum_values entry `{}` must be unique",
                descriptor.key, enum_value
            ));
        }
    }
    if !descriptor
        .enum_values
        .iter()
        .any(|enum_value| enum_value == &descriptor.default_value)
    {
        return invalid_plugin_option(format!(
            "{} enum default_value `{}` must be declared in enum_values",
            descriptor.key, descriptor.default_value
        ));
    }
    Ok(())
}

fn validate_plugin_option_enum_token(
    option_key: &str,
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_option_field(field_name, value)?;
    if !value.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
    }) {
        return invalid_plugin_option(format!(
            "{option_key} enum {field_name} value `{value}` must contain only lowercase ASCII letters, digits, underscores, or hyphens"
        ));
    }
    Ok(())
}

fn validate_plugin_option_field(
    field_name: &str,
    value: &str,
) -> Result<(), RuntimeExtensionRegistryError> {
    if value.trim().is_empty() || value.trim() != value {
        return invalid_plugin_option(format!(
            "{field_name} `{value}` must be non-empty and trimmed"
        ));
    }
    Ok(())
}

fn invalid_plugin_option<T>(message: String) -> Result<T, RuntimeExtensionRegistryError> {
    Err(RuntimeExtensionRegistryError::InvalidPluginOption(message))
}
