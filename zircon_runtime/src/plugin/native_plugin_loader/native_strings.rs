use std::ffi::{c_char, CStr};

use crate::plugin::PluginPackageManifest;

pub(super) type NativeStringResult<T> = std::result::Result<T, NativeStringError>;

#[derive(Debug)]
pub(super) enum NativeStringError {
    MissingRequiredField {
        field_name: String,
    },
    InvalidPackageManifest {
        message: String,
        source: toml::de::Error,
    },
}

impl std::fmt::Display for NativeStringError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequiredField { field_name } => write!(
                formatter,
                "native plugin descriptor field {field_name} is null or invalid"
            ),
            Self::InvalidPackageManifest { message, source } => {
                write!(formatter, "{message}: {source}")
            }
        }
    }
}

impl std::error::Error for NativeStringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingRequiredField { .. } => None,
            Self::InvalidPackageManifest { source, .. } => Some(source),
        }
    }
}

pub(super) fn native_symbol_name(symbol_name: &str) -> Vec<u8> {
    let mut bytes = symbol_name.as_bytes().to_vec();
    if !bytes.ends_with(&[0]) {
        bytes.push(0);
    }
    bytes
}

pub(super) unsafe fn read_required_c_string(
    value: *const c_char,
    field_name: &str,
) -> NativeStringResult<String> {
    read_optional_c_string(value).ok_or_else(|| NativeStringError::MissingRequiredField {
        field_name: field_name.to_string(),
    })
}

pub(super) unsafe fn read_optional_c_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    CStr::from_ptr(value).to_str().ok().map(str::to_string)
}

pub(super) fn package_manifest_from_toml(
    manifest_toml: &str,
    invalid_message: &str,
) -> NativeStringResult<Option<PluginPackageManifest>> {
    if manifest_toml.trim().is_empty() {
        return Ok(None);
    }
    toml::from_str::<PluginPackageManifest>(manifest_toml)
        .map(Some)
        .map_err(|source| NativeStringError::InvalidPackageManifest {
            message: invalid_message.to_string(),
            source,
        })
}

pub(super) fn parse_native_string_list(value: &str) -> Vec<String> {
    let mut entries = Vec::new();
    for entry in value
        .split(|character| matches!(character, '\n' | ',' | ';'))
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        if !entries.iter().any(|existing| existing == entry) {
            entries.push(entry.to_string());
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_required_c_string_reports_missing_field_with_typed_error() {
        let error = unsafe { read_required_c_string(std::ptr::null(), "plugin_id") }
            .expect_err("null required field should report typed string error");

        match error {
            NativeStringError::MissingRequiredField { field_name } => {
                assert_eq!(field_name, "plugin_id");
            }
            NativeStringError::InvalidPackageManifest { .. } => {
                panic!("null required field should not report package manifest parse error")
            }
        }
    }

    #[test]
    fn native_string_typed_error_preserves_package_manifest_message() {
        let error =
            package_manifest_from_toml("not = [", "native plugin package manifest is invalid")
                .expect_err("invalid TOML should report typed package manifest error");

        assert!(
            error
                .to_string()
                .starts_with("native plugin package manifest is invalid: "),
            "typed package manifest error should preserve existing diagnostic prefix"
        );
        assert!(
            std::error::Error::source(&error).is_some(),
            "package manifest error should preserve TOML source"
        );
    }
}
