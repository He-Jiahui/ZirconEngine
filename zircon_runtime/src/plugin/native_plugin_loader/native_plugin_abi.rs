use std::ffi::{c_char, CStr};

use libloading::Library;

use crate::{PluginModuleKind, PluginPackageManifest};

pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = 1;
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL: &[u8] = b"zircon_native_plugin_descriptor_v1\0";

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginAbiV1 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV1 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginDescriptor {
    pub abi_version: u32,
    pub plugin_id: String,
    pub package_manifest: Option<PluginPackageManifest>,
    pub runtime_entry_name: Option<String>,
    pub editor_entry_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginEntryReport {
    pub plugin_id: String,
    pub module_kind: PluginModuleKind,
    pub package_manifest: Option<PluginPackageManifest>,
    pub diagnostics: Vec<String>,
}

type NativePluginDescriptorFn = unsafe extern "C" fn() -> *const NativePluginAbiV1;
type NativePluginEntryFn = unsafe extern "C" fn() -> *const NativePluginEntryReportV1;

pub(super) unsafe fn probe_native_plugin_descriptor(
    library: &Library,
) -> Result<Option<NativePluginDescriptor>, String> {
    let symbol =
        match library.get::<NativePluginDescriptorFn>(ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL) {
            Ok(symbol) => symbol,
            Err(_) => return Ok(None),
        };
    let descriptor = symbol();
    if descriptor.is_null() {
        return Err("native plugin descriptor symbol returned null".to_string());
    }
    NativePluginDescriptor::from_abi(&*descriptor).map(Some)
}

pub(super) unsafe fn call_native_plugin_entry(
    library: &Library,
    symbol_name: &str,
    plugin_id: &str,
    module_kind: PluginModuleKind,
) -> Result<NativePluginEntryReport, String> {
    let symbol_name = native_symbol_name(symbol_name);
    let symbol = library
        .get::<NativePluginEntryFn>(&symbol_name[..])
        .map_err(|error| format!("native plugin entry symbol is missing: {error}"))?;
    let report = symbol();
    if report.is_null() {
        return Err("native plugin entry returned null".to_string());
    }
    NativePluginEntryReport::from_abi(plugin_id, module_kind, &*report)
}

impl NativePluginDescriptor {
    unsafe fn from_abi(abi: &NativePluginAbiV1) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION {
            return Err(format!(
                "unsupported native plugin ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION
            ));
        }
        let plugin_id = read_required_c_string(abi.plugin_id, "plugin_id")?;
        let manifest_toml = read_optional_c_string(abi.package_manifest_toml).unwrap_or_default();
        let package_manifest = if manifest_toml.trim().is_empty() {
            None
        } else {
            Some(
                toml::from_str::<PluginPackageManifest>(&manifest_toml).map_err(|error| {
                    format!("native plugin package manifest is invalid: {error}")
                })?,
            )
        };
        Ok(Self {
            abi_version: abi.abi_version,
            plugin_id,
            package_manifest,
            runtime_entry_name: read_optional_c_string(abi.runtime_entry_name),
            editor_entry_name: read_optional_c_string(abi.editor_entry_name),
        })
    }
}

impl NativePluginEntryReport {
    unsafe fn from_abi(
        plugin_id: &str,
        module_kind: PluginModuleKind,
        abi: &NativePluginEntryReportV1,
    ) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION {
            return Err(format!(
                "unsupported native plugin entry ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION
            ));
        }
        let manifest_toml = read_optional_c_string(abi.package_manifest_toml).unwrap_or_default();
        let package_manifest = if manifest_toml.trim().is_empty() {
            None
        } else {
            Some(
                toml::from_str::<PluginPackageManifest>(&manifest_toml).map_err(|error| {
                    format!("native plugin entry package manifest is invalid: {error}")
                })?,
            )
        };
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            module_kind,
            package_manifest,
            diagnostics: read_optional_c_string(abi.diagnostics)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
        })
    }
}

fn native_symbol_name(symbol_name: &str) -> Vec<u8> {
    let mut bytes = symbol_name.as_bytes().to_vec();
    if !bytes.ends_with(&[0]) {
        bytes.push(0);
    }
    bytes
}

unsafe fn read_required_c_string(value: *const c_char, field_name: &str) -> Result<String, String> {
    read_optional_c_string(value)
        .ok_or_else(|| format!("native plugin descriptor field {field_name} is null or invalid"))
}

unsafe fn read_optional_c_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    CStr::from_ptr(value).to_str().ok().map(str::to_string)
}
