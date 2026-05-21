use std::ffi::{c_char, CStr};

use crate::plugin::PluginPackageManifest;

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
) -> Result<String, String> {
    read_optional_c_string(value)
        .ok_or_else(|| format!("native plugin descriptor field {field_name} is null or invalid"))
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
) -> Result<Option<PluginPackageManifest>, String> {
    if manifest_toml.trim().is_empty() {
        return Ok(None);
    }
    toml::from_str::<PluginPackageManifest>(manifest_toml)
        .map(Some)
        .map_err(|error| format!("{invalid_message}: {error}"))
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
