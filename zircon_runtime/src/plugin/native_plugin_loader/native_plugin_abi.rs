use std::ffi::CString;

use libloading::Library;

use crate::plugin::{PluginModuleKind, PluginPackageManifest};

use super::abi_declarations::{
    NativePluginAbiV1, NativePluginAbiV2, NativePluginAbiV3, NativePluginEntryReportV1,
    NativePluginEntryReportV2, NativePluginEntryReportV3, NativePluginHostFunctionTableV2,
    NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V1, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V2,
    ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3,
};
use super::behavior_calls::NativePluginBehavior;
use super::behavior_validation::NativePluginBehaviorValidationReport;
use super::host_callbacks::{
    granted_capabilities_for_entry, native_host_abi_version_v2, native_host_abi_version_v3,
    native_host_diagnostic_v3, native_host_has_capability_v2, native_host_has_capability_v3,
    native_host_log_v3, register_native_host_callback_capture,
    take_native_host_callback_diagnostics,
};
use super::native_strings::{
    native_symbol_name, package_manifest_from_toml, parse_native_string_list,
    read_optional_c_string, read_required_c_string,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginDescriptor {
    pub abi_version: u32,
    pub plugin_id: String,
    pub package_manifest: Option<PluginPackageManifest>,
    pub runtime_entry_name: Option<String>,
    pub editor_entry_name: Option<String>,
    pub requested_capabilities: Vec<String>,
}

#[derive(Debug)]
pub struct NativePluginEntryReport {
    pub plugin_id: String,
    pub module_kind: PluginModuleKind,
    pub package_manifest: Option<PluginPackageManifest>,
    pub diagnostics: Vec<String>,
    pub negotiated_capabilities: Vec<String>,
    pub(super) behavior: Option<NativePluginBehavior>,
    pub behavior_validation: NativePluginBehaviorValidationReport,
}

type NativePluginDescriptorFnV1 = unsafe extern "C" fn() -> *const NativePluginAbiV1;
type NativePluginDescriptorFnV2 = unsafe extern "C" fn() -> *const NativePluginAbiV2;
type NativePluginDescriptorFnV3 = unsafe extern "C" fn() -> *const NativePluginAbiV3;
type NativePluginEntryFnV1 = unsafe extern "C" fn() -> *const NativePluginEntryReportV1;
type NativePluginEntryFnV2 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV2,
) -> *const NativePluginEntryReportV2;
type NativePluginEntryFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
) -> *const NativePluginEntryReportV3;

pub(super) unsafe fn probe_native_plugin_descriptor(
    library: &Library,
) -> Result<Option<NativePluginDescriptor>, String> {
    if let Ok(symbol) =
        library.get::<NativePluginDescriptorFnV3>(ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3)
    {
        let descriptor = symbol();
        if descriptor.is_null() {
            return Err("native plugin ABI v3 descriptor symbol returned null".to_string());
        }
        return NativePluginDescriptor::from_abi_v3(&*descriptor).map(Some);
    }

    if let Ok(symbol) =
        library.get::<NativePluginDescriptorFnV2>(ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V2)
    {
        let descriptor = symbol();
        if descriptor.is_null() {
            return Err("native plugin ABI v2 descriptor symbol returned null".to_string());
        }
        return NativePluginDescriptor::from_abi_v2(&*descriptor).map(Some);
    }

    let symbol = match library
        .get::<NativePluginDescriptorFnV1>(ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V1)
    {
        Ok(symbol) => symbol,
        Err(_) => return Ok(None),
    };
    let descriptor = symbol();
    if descriptor.is_null() {
        return Err("native plugin ABI v1 descriptor symbol returned null".to_string());
    }
    NativePluginDescriptor::from_abi_v1(&*descriptor).map(Some)
}

pub(super) unsafe fn call_native_plugin_entry(
    library: &Library,
    symbol_name: &str,
    plugin_id: &str,
    module_kind: PluginModuleKind,
    descriptor: &NativePluginDescriptor,
) -> Result<NativePluginEntryReport, String> {
    let symbol_name = native_symbol_name(symbol_name);
    if descriptor.abi_version == ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
        let symbol = library
            .get::<NativePluginEntryFnV3>(&symbol_name[..])
            .map_err(|error| format!("native plugin entry symbol is missing: {error}"))?;
        let granted_capabilities = granted_capabilities_for_entry(descriptor, module_kind);
        let granted_capabilities = CString::new(granted_capabilities.join("\n")).map_err(|_| {
            "native plugin requested capability contained an interior NUL".to_string()
        })?;
        let host_handle = register_native_host_callback_capture();
        let host_functions = NativePluginHostFunctionTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            host_handle,
            granted_capabilities: granted_capabilities.as_ptr(),
            host_abi_version: Some(native_host_abi_version_v3),
            host_has_capability: Some(native_host_has_capability_v3),
            host_log: Some(native_host_log_v3),
            host_diagnostic: Some(native_host_diagnostic_v3),
        };
        let report = symbol(&host_functions);
        let callback_diagnostics = take_native_host_callback_diagnostics(host_handle);
        if report.is_null() {
            return Err("native plugin entry returned null".to_string());
        }
        let mut report = NativePluginEntryReport::from_abi_v3(plugin_id, module_kind, &*report)?;
        report.diagnostics.extend(callback_diagnostics);
        Ok(report)
    } else if descriptor.abi_version == ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2 {
        let symbol = library
            .get::<NativePluginEntryFnV2>(&symbol_name[..])
            .map_err(|error| format!("native plugin entry symbol is missing: {error}"))?;
        let granted_capabilities = granted_capabilities_for_entry(descriptor, module_kind);
        let granted_capabilities = CString::new(granted_capabilities.join("\n")).map_err(|_| {
            "native plugin requested capability contained an interior NUL".to_string()
        })?;
        let host_functions = NativePluginHostFunctionTableV2 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2,
            host_handle: 1,
            granted_capabilities: granted_capabilities.as_ptr(),
            host_abi_version: Some(native_host_abi_version_v2),
            host_has_capability: Some(native_host_has_capability_v2),
        };
        let report = symbol(&host_functions);
        if report.is_null() {
            return Err("native plugin entry returned null".to_string());
        }
        NativePluginEntryReport::from_abi_v2(plugin_id, module_kind, &*report)
    } else {
        let symbol = library
            .get::<NativePluginEntryFnV1>(&symbol_name[..])
            .map_err(|error| format!("native plugin entry symbol is missing: {error}"))?;
        let report = symbol();
        if report.is_null() {
            return Err("native plugin entry returned null".to_string());
        }
        NativePluginEntryReport::from_abi_v1(plugin_id, module_kind, &*report)
    }
}

impl NativePluginDescriptor {
    unsafe fn from_abi_v1(abi: &NativePluginAbiV1) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1 {
            return Err(format!(
                "unsupported native plugin ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1
            ));
        }
        let plugin_id = read_required_c_string(abi.plugin_id, "plugin_id")?;
        Ok(Self {
            abi_version: abi.abi_version,
            plugin_id,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin package manifest is invalid",
            )?,
            runtime_entry_name: read_optional_c_string(abi.runtime_entry_name),
            editor_entry_name: read_optional_c_string(abi.editor_entry_name),
            requested_capabilities: Vec::new(),
        })
    }

    unsafe fn from_abi_v2(abi: &NativePluginAbiV2) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2 {
            return Err(format!(
                "unsupported native plugin ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2
            ));
        }
        let plugin_id = read_required_c_string(abi.plugin_id, "plugin_id")?;
        Ok(Self {
            abi_version: abi.abi_version,
            plugin_id,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin package manifest is invalid",
            )?,
            runtime_entry_name: read_optional_c_string(abi.runtime_entry_name),
            editor_entry_name: read_optional_c_string(abi.editor_entry_name),
            requested_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.requested_capabilities).unwrap_or_default(),
            ),
        })
    }

    unsafe fn from_abi_v3(abi: &NativePluginAbiV3) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
            return Err(format!(
                "unsupported native plugin ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
            ));
        }
        let plugin_id = read_required_c_string(abi.plugin_id, "plugin_id")?;
        Ok(Self {
            abi_version: abi.abi_version,
            plugin_id,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin package manifest is invalid",
            )?,
            runtime_entry_name: read_optional_c_string(abi.runtime_entry_name),
            editor_entry_name: read_optional_c_string(abi.editor_entry_name),
            requested_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.requested_capabilities).unwrap_or_default(),
            ),
        })
    }
}

impl NativePluginEntryReport {
    unsafe fn from_abi_v1(
        plugin_id: &str,
        module_kind: PluginModuleKind,
        abi: &NativePluginEntryReportV1,
    ) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1 {
            return Err(format!(
                "unsupported native plugin entry ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1
            ));
        }
        let behavior = None;
        let behavior_validation = NativePluginBehaviorValidationReport::from_behavior(
            plugin_id,
            module_kind,
            abi.abi_version,
            behavior.as_ref(),
        );
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            module_kind,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin entry package manifest is invalid",
            )?,
            diagnostics: entry_diagnostics(abi.diagnostics),
            negotiated_capabilities: Vec::new(),
            behavior,
            behavior_validation,
        })
    }

    unsafe fn from_abi_v2(
        plugin_id: &str,
        module_kind: PluginModuleKind,
        abi: &NativePluginEntryReportV2,
    ) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2 {
            return Err(format!(
                "unsupported native plugin entry ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2
            ));
        }
        let behavior = if abi.behavior.is_null() {
            None
        } else {
            Some(NativePluginBehavior::from_abi_v2(&*abi.behavior)?)
        };
        let behavior_validation = NativePluginBehaviorValidationReport::from_behavior(
            plugin_id,
            module_kind,
            abi.abi_version,
            behavior.as_ref(),
        );
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            module_kind,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin entry package manifest is invalid",
            )?,
            diagnostics: entry_diagnostics(abi.diagnostics),
            negotiated_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.negotiated_capabilities).unwrap_or_default(),
            ),
            behavior_validation,
            behavior,
        })
    }

    unsafe fn from_abi_v3(
        plugin_id: &str,
        module_kind: PluginModuleKind,
        abi: &NativePluginEntryReportV3,
    ) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
            return Err(format!(
                "unsupported native plugin entry ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
            ));
        }
        let behavior = if abi.behavior.is_null() {
            None
        } else {
            Some(NativePluginBehavior::from_abi_v3(&*abi.behavior)?)
        };
        let behavior_validation = NativePluginBehaviorValidationReport::from_behavior(
            plugin_id,
            module_kind,
            abi.abi_version,
            behavior.as_ref(),
        );
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            module_kind,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin entry package manifest is invalid",
            )?,
            diagnostics: entry_diagnostics(abi.diagnostics),
            negotiated_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.negotiated_capabilities).unwrap_or_default(),
            ),
            behavior_validation,
            behavior,
        })
    }
}

unsafe fn entry_diagnostics(diagnostics: *const std::ffi::c_char) -> Vec<String> {
    read_optional_c_string(diagnostics)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}
