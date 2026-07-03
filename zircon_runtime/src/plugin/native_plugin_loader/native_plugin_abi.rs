use std::ffi::{CString, NulError};

use libloading::Library;

use crate::plugin::{PluginModuleKind, PluginPackageManifest};

use super::abi_declarations::{
    NativePluginAbiV3, NativePluginEntryReportV3, NativePluginHostFunctionTableV3,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3, ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3,
};
use super::behavior_calls::NativePluginBehavior;
use super::behavior_validation::NativePluginBehaviorValidationReport;
use super::bridge_method_abi::bridge_method_bindings_from_abi_v3;
use super::bridge_method_bindings::NativeBridgeMethodBinding;
use super::host_callbacks::{
    granted_capabilities_for_entry, native_host_abi_version_v3, native_host_diagnostic_v3,
    native_host_has_capability_v3, native_host_log_v3, register_native_host_callback_capture,
    take_native_host_callback_diagnostics,
};
use super::native_strings::{
    native_symbol_name, package_manifest_from_toml, parse_native_string_list,
    read_optional_c_string, read_required_c_string, NativeStringError,
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
    pub bridge_method_bindings: Vec<NativeBridgeMethodBinding>,
    pub(super) behavior: Option<NativePluginBehavior>,
    pub behavior_validation: NativePluginBehaviorValidationReport,
}

type NativePluginDescriptorFnV3 = unsafe extern "C" fn() -> *const NativePluginAbiV3;
type NativePluginEntryFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
) -> *const NativePluginEntryReportV3;

type NativePluginDescriptorAbiResult<T> = std::result::Result<T, NativePluginDescriptorAbiError>;
type NativePluginEntryAbiResult<T> = std::result::Result<T, NativePluginEntryAbiError>;

#[derive(Debug)]
enum NativePluginDescriptorAbiError {
    NullDescriptorSymbol,
    UnsupportedAbiVersion {
        actual: u32,
        expected: u32,
    },
    InvalidRequiredField {
        field_name: &'static str,
        source: NativeStringError,
    },
    InvalidPackageManifest {
        source: NativeStringError,
    },
}

impl std::fmt::Display for NativePluginDescriptorAbiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullDescriptorSymbol => {
                formatter.write_str("native plugin ABI v3 descriptor symbol returned null")
            }
            Self::UnsupportedAbiVersion { actual, expected } => write!(
                formatter,
                "unsupported native plugin ABI version {actual}; expected {expected}"
            ),
            Self::InvalidRequiredField { source, .. } => write!(formatter, "{source}"),
            Self::InvalidPackageManifest { source } => write!(formatter, "{source}"),
        }
    }
}

impl std::error::Error for NativePluginDescriptorAbiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NullDescriptorSymbol | Self::UnsupportedAbiVersion { .. } => None,
            Self::InvalidRequiredField { source, .. } => Some(source),
            Self::InvalidPackageManifest { source } => Some(source),
        }
    }
}

#[derive(Debug)]
enum NativePluginEntryAbiError {
    UnsupportedDescriptorAbiVersion {
        actual: u32,
        expected: u32,
    },
    MissingEntrySymbol {
        source: libloading::Error,
    },
    InvalidGrantedCapabilities {
        source: NulError,
    },
    NullEntryReport,
    UnsupportedEntryAbiVersion {
        actual: u32,
        expected: u32,
    },
    InvalidBehavior {
        source: super::behavior_calls::NativePluginBehaviorError,
    },
    InvalidPackageManifest {
        source: NativeStringError,
    },
    InvalidBridgeMethods {
        source: super::bridge_method_abi::NativeBridgeMethodAbiError,
    },
}

impl std::fmt::Display for NativePluginEntryAbiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedDescriptorAbiVersion { actual, expected } => write!(
                formatter,
                "unsupported native plugin ABI version {actual}; expected {expected}"
            ),
            Self::MissingEntrySymbol { source } => {
                write!(formatter, "native plugin entry symbol is missing: {source}")
            }
            Self::InvalidGrantedCapabilities { .. } => {
                formatter.write_str("native plugin requested capability contained an interior NUL")
            }
            Self::NullEntryReport => formatter.write_str("native plugin entry returned null"),
            Self::UnsupportedEntryAbiVersion { actual, expected } => write!(
                formatter,
                "unsupported native plugin entry ABI version {actual}; expected {expected}"
            ),
            Self::InvalidBehavior { source } => write!(formatter, "{source}"),
            Self::InvalidPackageManifest { source } => write!(formatter, "{source}"),
            Self::InvalidBridgeMethods { source } => write!(formatter, "{source}"),
        }
    }
}

impl std::error::Error for NativePluginEntryAbiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedDescriptorAbiVersion { .. }
            | Self::NullEntryReport
            | Self::UnsupportedEntryAbiVersion { .. } => None,
            Self::MissingEntrySymbol { source } => Some(source),
            Self::InvalidGrantedCapabilities { source } => Some(source),
            Self::InvalidBehavior { source } => Some(source),
            Self::InvalidPackageManifest { source } => Some(source),
            Self::InvalidBridgeMethods { source } => Some(source),
        }
    }
}

pub(super) unsafe fn probe_native_plugin_descriptor(
    library: &Library,
) -> Result<Option<NativePluginDescriptor>, String> {
    let symbol = match library
        .get::<NativePluginDescriptorFnV3>(ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3)
    {
        Ok(symbol) => symbol,
        Err(_) => return Ok(None),
    };
    let descriptor = symbol();
    if descriptor.is_null() {
        return Err(NativePluginDescriptorAbiError::NullDescriptorSymbol.to_string());
    }
    NativePluginDescriptor::from_abi_v3(&*descriptor)
        .map(Some)
        .map_err(|error| error.to_string())
}

pub(super) unsafe fn call_native_plugin_entry(
    library: &Library,
    symbol_name: &str,
    plugin_id: &str,
    module_kind: PluginModuleKind,
    descriptor: &NativePluginDescriptor,
) -> Result<NativePluginEntryReport, String> {
    unsafe {
        call_native_plugin_entry_result(library, symbol_name, plugin_id, module_kind, descriptor)
    }
    .map_err(|error| error.to_string())
}

unsafe fn call_native_plugin_entry_result(
    library: &Library,
    symbol_name: &str,
    plugin_id: &str,
    module_kind: PluginModuleKind,
    descriptor: &NativePluginDescriptor,
) -> NativePluginEntryAbiResult<NativePluginEntryReport> {
    let symbol_name = native_symbol_name(symbol_name);
    if descriptor.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
        return Err(NativePluginEntryAbiError::UnsupportedDescriptorAbiVersion {
            actual: descriptor.abi_version,
            expected: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        });
    }
    let symbol = library
        .get::<NativePluginEntryFnV3>(&symbol_name[..])
        .map_err(|source| NativePluginEntryAbiError::MissingEntrySymbol { source })?;
    let granted_capabilities = granted_capabilities_for_entry(descriptor, module_kind);
    let granted_capabilities = CString::new(granted_capabilities.join("\n"))
        .map_err(|source| NativePluginEntryAbiError::InvalidGrantedCapabilities { source })?;
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
        return Err(NativePluginEntryAbiError::NullEntryReport);
    }
    let mut report = NativePluginEntryReport::from_abi_v3(plugin_id, module_kind, &*report)?;
    report.diagnostics.extend(callback_diagnostics);
    Ok(report)
}

impl NativePluginDescriptor {
    unsafe fn from_abi_v3(abi: &NativePluginAbiV3) -> NativePluginDescriptorAbiResult<Self> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
            return Err(NativePluginDescriptorAbiError::UnsupportedAbiVersion {
                actual: abi.abi_version,
                expected: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            });
        }
        let plugin_id = read_required_descriptor_field(abi.plugin_id, "plugin_id")?;
        Ok(Self {
            abi_version: abi.abi_version,
            plugin_id,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin package manifest is invalid",
            )
            .map_err(|source| NativePluginDescriptorAbiError::InvalidPackageManifest { source })?,
            runtime_entry_name: read_optional_c_string(abi.runtime_entry_name),
            editor_entry_name: read_optional_c_string(abi.editor_entry_name),
            requested_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.requested_capabilities).unwrap_or_default(),
            ),
        })
    }
}

unsafe fn read_required_descriptor_field(
    value: *const std::ffi::c_char,
    field_name: &'static str,
) -> NativePluginDescriptorAbiResult<String> {
    unsafe { read_required_c_string(value, field_name) }.map_err(|source| {
        NativePluginDescriptorAbiError::InvalidRequiredField { field_name, source }
    })
}

impl NativePluginEntryReport {
    unsafe fn from_abi_v3(
        plugin_id: &str,
        module_kind: PluginModuleKind,
        abi: &NativePluginEntryReportV3,
    ) -> NativePluginEntryAbiResult<Self> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
            return Err(NativePluginEntryAbiError::UnsupportedEntryAbiVersion {
                actual: abi.abi_version,
                expected: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            });
        }
        let behavior = if abi.behavior.is_null() {
            None
        } else {
            Some(
                NativePluginBehavior::from_abi_v3(&*abi.behavior)
                    .map_err(|source| NativePluginEntryAbiError::InvalidBehavior { source })?,
            )
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
            )
            .map_err(|source| NativePluginEntryAbiError::InvalidPackageManifest { source })?,
            diagnostics: entry_diagnostics(abi.diagnostics),
            negotiated_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.negotiated_capabilities).unwrap_or_default(),
            ),
            bridge_method_bindings: bridge_method_bindings_from_abi_v3(abi.bridge_methods)
                .map_err(|source| NativePluginEntryAbiError::InvalidBridgeMethods { source })?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_entry_abi_error_preserves_granted_capability_source() {
        let source = CString::new("native\0capability")
            .expect_err("interior NUL should be rejected by CString");
        let error = NativePluginEntryAbiError::InvalidGrantedCapabilities { source };

        assert_eq!(
            error.to_string(),
            "native plugin requested capability contained an interior NUL"
        );
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn native_entry_abi_error_preserves_unsupported_entry_message() {
        let error = NativePluginEntryAbiError::UnsupportedEntryAbiVersion {
            actual: 2,
            expected: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        };

        assert_eq!(
            error.to_string(),
            format!(
                "unsupported native plugin entry ABI version 2; expected {}",
                ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
            )
        );
        assert!(std::error::Error::source(&error).is_none());
    }
}
