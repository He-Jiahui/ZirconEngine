use std::collections::BTreeMap;
use std::ffi::{c_char, CStr, CString};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use libloading::Library;

use crate::{
    plugin::PluginModuleKind, plugin::PluginModuleManifest, plugin::PluginPackageManifest,
};

pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1: u32 = 1;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2: u32 = 2;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3: u32 = 3;
pub const ZIRCON_NATIVE_PLUGIN_ABI_VERSION: u32 = ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V1: &[u8] =
    b"zircon_native_plugin_descriptor_v1\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V2: &[u8] =
    b"zircon_native_plugin_descriptor_v2\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3: &[u8] =
    b"zircon_native_plugin_descriptor_v3\0";
pub const ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL: &[u8] = ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3;

pub const ZIRCON_NATIVE_PLUGIN_STATUS_OK: u32 = 0;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_ERROR: u32 = 1;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_DENIED: u32 = 2;
pub const ZIRCON_NATIVE_PLUGIN_STATUS_PANIC: u32 = 3;

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
pub struct NativePluginAbiV2 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
    pub requested_capabilities: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginAbiV3 {
    pub abi_version: u32,
    pub plugin_id: *const c_char,
    pub package_manifest_toml: *const c_char,
    pub runtime_entry_name: *const c_char,
    pub editor_entry_name: *const c_char,
    pub requested_capabilities: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginSchemaVersionsV3 {
    pub state_schema_version: u32,
    pub command_manifest_schema: *const c_char,
    pub event_manifest_schema: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV1 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV2 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
    pub negotiated_capabilities: *const c_char,
    pub behavior: *const NativePluginBehaviorV2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginEntryReportV3 {
    pub abi_version: u32,
    pub package_manifest_toml: *const c_char,
    pub diagnostics: *const c_char,
    pub negotiated_capabilities: *const c_char,
    pub behavior: *const NativePluginBehaviorV3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginHostFunctionTableV2 {
    pub abi_version: u32,
    pub host_handle: u64,
    pub granted_capabilities: *const c_char,
    pub host_abi_version: Option<unsafe extern "C" fn() -> u32>,
    pub host_has_capability:
        Option<unsafe extern "C" fn(*const NativePluginHostFunctionTableV2, *const c_char) -> u32>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginHostFunctionTableV3 {
    pub abi_version: u32,
    pub host_handle: u64,
    pub granted_capabilities: *const c_char,
    pub host_abi_version: Option<unsafe extern "C" fn() -> u32>,
    pub host_has_capability: Option<NativePluginHostHasCapabilityFnV3>,
    pub host_log: Option<NativePluginHostLogFnV3>,
    pub host_diagnostic: Option<NativePluginHostDiagnosticFnV3>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginByteSliceV2 {
    pub data: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginOwnedByteBufferV2 {
    pub data: *mut u8,
    pub len: usize,
    pub capacity: usize,
    pub owner_token: u64,
    pub free: Option<NativePluginFreeBytesFnV2>,
}

impl NativePluginOwnedByteBufferV2 {
    #[cfg(test)]
    pub(crate) fn new_for_test(
        data: *mut u8,
        len: usize,
        capacity: usize,
        owner_token: u64,
        free: Option<NativePluginFreeBytesFnV2>,
    ) -> Self {
        Self {
            data,
            len,
            capacity,
            owner_token,
            free,
        }
    }

    fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
            owner_token: 0,
            free: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginCallbackStatusV2 {
    pub code: u32,
    pub diagnostics: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBehaviorV2 {
    pub abi_version: u32,
    pub is_stateless: u32,
    pub command_manifest: *const c_char,
    pub event_manifest: *const c_char,
    pub invoke_command: Option<NativePluginInvokeCommandFnV2>,
    pub save_state: Option<NativePluginSaveStateFnV2>,
    pub restore_state: Option<NativePluginRestoreStateFnV2>,
    pub unload: Option<NativePluginUnloadFnV2>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativePluginBehaviorV3 {
    pub abi_version: u32,
    pub is_stateless: u32,
    pub schema_versions: NativePluginSchemaVersionsV3,
    pub command_manifest: *const c_char,
    pub event_manifest: *const c_char,
    pub invoke_command: Option<NativePluginInvokeCommandFnV3>,
    pub save_state: Option<NativePluginSaveStateFnV3>,
    pub restore_state: Option<NativePluginRestoreStateFnV3>,
    pub unload: Option<NativePluginUnloadFnV3>,
}

pub type NativePluginFreeBytesFnV2 =
    unsafe extern "C" fn(NativePluginOwnedByteBufferV2) -> NativePluginCallbackStatusV2;
pub type NativePluginInvokeCommandFnV2 = unsafe extern "C" fn(
    *const c_char,
    NativePluginByteSliceV2,
    *mut NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2;
pub type NativePluginSaveStateFnV2 =
    unsafe extern "C" fn(*mut NativePluginOwnedByteBufferV2) -> NativePluginCallbackStatusV2;
pub type NativePluginRestoreStateFnV2 =
    unsafe extern "C" fn(NativePluginByteSliceV2) -> NativePluginCallbackStatusV2;
pub type NativePluginUnloadFnV2 = unsafe extern "C" fn() -> NativePluginCallbackStatusV2;
pub type NativePluginByteSliceV3 = NativePluginByteSliceV2;
pub type NativePluginOwnedByteBufferV3 = NativePluginOwnedByteBufferV2;
pub type NativePluginCallbackStatusV3 = NativePluginCallbackStatusV2;
pub type NativePluginInvokeCommandFnV3 = NativePluginInvokeCommandFnV2;
pub type NativePluginSaveStateFnV3 = NativePluginSaveStateFnV2;
pub type NativePluginRestoreStateFnV3 = NativePluginRestoreStateFnV2;
pub type NativePluginUnloadFnV3 = NativePluginUnloadFnV2;
pub type NativePluginHostHasCapabilityFnV3 =
    unsafe extern "C" fn(*const NativePluginHostFunctionTableV3, *const c_char) -> u32;
pub type NativePluginHostLogFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
    u32,
    *const c_char,
    *const c_char,
) -> u32;
pub type NativePluginHostDiagnosticFnV3 = unsafe extern "C" fn(
    *const NativePluginHostFunctionTableV3,
    *const c_char,
    f64,
    *const c_char,
    *const c_char,
) -> u32;

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
}

#[derive(Debug)]
pub(super) struct NativePluginBehavior {
    pub is_stateless: bool,
    pub state_schema_version: u32,
    pub command_manifest_schema: Option<String>,
    pub event_manifest_schema: Option<String>,
    pub command_manifest: Option<String>,
    pub event_manifest: Option<String>,
    invoke_command: Option<NativePluginInvokeCommandFnV2>,
    save_state: Option<NativePluginSaveStateFnV2>,
    restore_state: Option<NativePluginRestoreStateFnV2>,
    unload: Option<NativePluginUnloadFnV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginBehaviorCallReport {
    pub status_code: u32,
    pub diagnostics: Vec<String>,
    pub payload: Option<Vec<u8>>,
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
        let callback_capture = take_native_host_callback_capture(host_handle);
        if report.is_null() {
            return Err("native plugin entry returned null".to_string());
        }
        let mut report = NativePluginEntryReport::from_abi_v3(plugin_id, module_kind, &*report)?;
        report
            .diagnostics
            .extend(callback_capture.into_entry_diagnostics());
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
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            module_kind,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin entry package manifest is invalid",
            )?,
            diagnostics: read_optional_c_string(abi.diagnostics)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
            negotiated_capabilities: Vec::new(),
            behavior: None,
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
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            module_kind,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin entry package manifest is invalid",
            )?,
            diagnostics: read_optional_c_string(abi.diagnostics)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
            negotiated_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.negotiated_capabilities).unwrap_or_default(),
            ),
            behavior: if abi.behavior.is_null() {
                None
            } else {
                Some(NativePluginBehavior::from_abi_v2(&*abi.behavior)?)
            },
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
        Ok(Self {
            plugin_id: plugin_id.to_string(),
            module_kind,
            package_manifest: package_manifest_from_toml(
                &read_optional_c_string(abi.package_manifest_toml).unwrap_or_default(),
                "native plugin entry package manifest is invalid",
            )?,
            diagnostics: read_optional_c_string(abi.diagnostics)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
            negotiated_capabilities: parse_native_string_list(
                &read_optional_c_string(abi.negotiated_capabilities).unwrap_or_default(),
            ),
            behavior: if abi.behavior.is_null() {
                None
            } else {
                Some(NativePluginBehavior::from_abi_v3(&*abi.behavior)?)
            },
        })
    }
}

impl NativePluginBehavior {
    unsafe fn from_abi_v2(abi: &NativePluginBehaviorV2) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2 {
            return Err(format!(
                "unsupported native plugin behavior ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2
            ));
        }
        Ok(Self {
            is_stateless: abi.is_stateless != 0,
            state_schema_version: 0,
            command_manifest_schema: None,
            event_manifest_schema: None,
            command_manifest: read_optional_c_string(abi.command_manifest),
            event_manifest: read_optional_c_string(abi.event_manifest),
            invoke_command: abi.invoke_command,
            save_state: abi.save_state,
            restore_state: abi.restore_state,
            unload: abi.unload,
        })
    }

    unsafe fn from_abi_v3(abi: &NativePluginBehaviorV3) -> Result<Self, String> {
        if abi.abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
            return Err(format!(
                "unsupported native plugin behavior ABI version {}; expected {}",
                abi.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
            ));
        }
        Ok(Self {
            is_stateless: abi.is_stateless != 0,
            state_schema_version: abi.schema_versions.state_schema_version,
            command_manifest_schema: read_optional_c_string(
                abi.schema_versions.command_manifest_schema,
            ),
            event_manifest_schema: read_optional_c_string(
                abi.schema_versions.event_manifest_schema,
            ),
            command_manifest: read_optional_c_string(abi.command_manifest),
            event_manifest: read_optional_c_string(abi.event_manifest),
            invoke_command: abi.invoke_command,
            save_state: abi.save_state,
            restore_state: abi.restore_state,
            unload: abi.unload,
        })
    }

    pub fn invoke_command(&self, name: &str, payload: &[u8]) -> NativePluginBehaviorCallReport {
        let Some(invoke_command) = self.invoke_command else {
            return missing_callback_report("invoke_command");
        };
        let Ok(name) = CString::new(name) else {
            return error_report("native plugin command name contained an interior NUL");
        };
        let mut output = NativePluginOwnedByteBufferV2::empty();
        let status = unsafe {
            invoke_command(
                name.as_ptr(),
                NativePluginByteSliceV2 {
                    data: payload.as_ptr(),
                    len: payload.len(),
                },
                &mut output,
            )
        };
        let mut report = NativePluginBehaviorCallReport::from_status(status);
        report.payload = take_owned_bytes(output, &mut report.diagnostics);
        report
    }

    pub fn save_state(&self) -> NativePluginBehaviorCallReport {
        let Some(save_state) = self.save_state else {
            return missing_callback_report("save_state");
        };
        let mut output = NativePluginOwnedByteBufferV2::empty();
        let status = unsafe { save_state(&mut output) };
        let mut report = NativePluginBehaviorCallReport::from_status(status);
        report.payload = take_owned_bytes(output, &mut report.diagnostics);
        report
    }

    pub fn restore_state(&self, state: &[u8]) -> NativePluginBehaviorCallReport {
        let Some(restore_state) = self.restore_state else {
            return missing_callback_report("restore_state");
        };
        let status = unsafe {
            restore_state(NativePluginByteSliceV2 {
                data: state.as_ptr(),
                len: state.len(),
            })
        };
        NativePluginBehaviorCallReport::from_status(status)
    }

    pub fn unload(&self) -> NativePluginBehaviorCallReport {
        let Some(unload) = self.unload else {
            return missing_callback_report("unload");
        };
        NativePluginBehaviorCallReport::from_status(unsafe { unload() })
    }
}

impl NativePluginBehaviorCallReport {
    fn from_status(status: NativePluginCallbackStatusV2) -> Self {
        Self {
            status_code: status.code,
            diagnostics: status_diagnostics(status),
            payload: None,
        }
    }
}

fn error_report(message: &str) -> NativePluginBehaviorCallReport {
    NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: vec![message.to_string()],
        payload: None,
    }
}

fn missing_callback_report(callback_name: &str) -> NativePluginBehaviorCallReport {
    error_report(&format!(
        "native plugin behavior callback {callback_name} is missing"
    ))
}

fn status_diagnostics(status: NativePluginCallbackStatusV2) -> Vec<String> {
    unsafe { read_optional_c_string(status.diagnostics) }
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn take_owned_bytes(
    output: NativePluginOwnedByteBufferV2,
    diagnostics: &mut Vec<String>,
) -> Option<Vec<u8>> {
    if output.data.is_null() {
        if output.len != 0 || output.capacity != 0 {
            diagnostics.push(format!(
                "native plugin owned buffer was malformed: null data with len {} and capacity {}",
                output.len, output.capacity
            ));
        }
        return None;
    }
    if output.len > output.capacity {
        diagnostics.push(format!(
            "native plugin owned buffer was malformed: len {} exceeds capacity {}",
            output.len, output.capacity
        ));
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(output.data.cast_const(), output.len) }.to_vec();
    let Some(free) = output.free else {
        diagnostics.push("native plugin owned buffer did not provide a free callback".to_string());
        return Some(bytes);
    };
    let free_status = unsafe { free(output) };
    if free_status.code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        diagnostics.extend(
            status_diagnostics(free_status)
                .into_iter()
                .map(|message| format!("native plugin owned buffer free failed: {message}")),
        );
    }
    Some(bytes)
}

unsafe extern "C" fn native_host_abi_version_v2() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2
}

unsafe extern "C" fn native_host_has_capability_v2(
    host_functions: *const NativePluginHostFunctionTableV2,
    capability: *const c_char,
) -> u32 {
    if host_functions.is_null() || capability.is_null() {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    }
    let Some(capability) = CStr::from_ptr(capability).to_str().ok() else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    };
    let Some(granted_capabilities) = read_optional_c_string((*host_functions).granted_capabilities)
    else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_DENIED;
    };
    if parse_native_string_list(&granted_capabilities)
        .iter()
        .any(|granted_capability| granted_capability == capability)
    {
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    } else {
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED
    }
}

unsafe extern "C" fn native_host_abi_version_v3() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
}

unsafe extern "C" fn native_host_has_capability_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capability: *const c_char,
) -> u32 {
    if host_functions.is_null() || capability.is_null() {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    }
    let Some(capability) = CStr::from_ptr(capability).to_str().ok() else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    };
    let Some(granted_capabilities) = read_optional_c_string((*host_functions).granted_capabilities)
    else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_DENIED;
    };
    if parse_native_string_list(&granted_capabilities)
        .iter()
        .any(|granted_capability| granted_capability == capability)
    {
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    } else {
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED
    }
}

unsafe extern "C" fn native_host_log_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    level: u32,
    target: *const c_char,
    message: *const c_char,
) -> u32 {
    let Some(mut capture) = native_host_callback_capture(host_functions) else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    };
    let Some(message) = read_optional_c_string(message) else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    };
    let target = read_optional_c_string(target).unwrap_or_else(|| "native_plugin".to_string());
    capture.logs.push(NativePluginHostLogRecord {
        level,
        target,
        message,
    });
    ZIRCON_NATIVE_PLUGIN_STATUS_OK
}

unsafe extern "C" fn native_host_diagnostic_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    path: *const c_char,
    value: f64,
    unit: *const c_char,
    tags: *const c_char,
) -> u32 {
    let Some(mut capture) = native_host_callback_capture(host_functions) else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    };
    let Some(path) = read_optional_c_string(path) else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    };
    capture.diagnostics.push(NativePluginHostDiagnosticRecord {
        path,
        value,
        unit: read_optional_c_string(unit),
        tags: parse_native_string_list(&read_optional_c_string(tags).unwrap_or_default()),
    });
    ZIRCON_NATIVE_PLUGIN_STATUS_OK
}

fn register_native_host_callback_capture() -> u64 {
    static NEXT_HOST_HANDLE: AtomicU64 = AtomicU64::new(2);
    let host_handle = NEXT_HOST_HANDLE.fetch_add(1, Ordering::Relaxed);
    let mut captures = lock_native_host_callback_captures();
    captures.insert(host_handle, NativePluginHostCallbackCapture::default());
    host_handle
}

fn take_native_host_callback_capture(host_handle: u64) -> NativePluginHostCallbackCapture {
    let mut captures = lock_native_host_callback_captures();
    captures.remove(&host_handle).unwrap_or_default()
}

unsafe fn native_host_callback_capture(
    host_functions: *const NativePluginHostFunctionTableV3,
) -> Option<NativePluginHostCallbackCaptureGuard<'static>> {
    if host_functions.is_null() {
        return None;
    }
    let host_handle = (*host_functions).host_handle;
    let captures = lock_native_host_callback_captures();
    if !captures.contains_key(&host_handle) {
        return None;
    }
    Some(NativePluginHostCallbackCaptureGuard {
        captures,
        host_handle,
    })
}

fn lock_native_host_callback_captures(
) -> std::sync::MutexGuard<'static, BTreeMap<u64, NativePluginHostCallbackCapture>> {
    native_host_callback_captures()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn native_host_callback_captures() -> &'static Mutex<BTreeMap<u64, NativePluginHostCallbackCapture>>
{
    static CAPTURES: OnceLock<Mutex<BTreeMap<u64, NativePluginHostCallbackCapture>>> =
        OnceLock::new();
    CAPTURES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

#[derive(Default)]
struct NativePluginHostCallbackCapture {
    logs: Vec<NativePluginHostLogRecord>,
    diagnostics: Vec<NativePluginHostDiagnosticRecord>,
}

impl NativePluginHostCallbackCapture {
    fn into_entry_diagnostics(self) -> Vec<String> {
        let mut diagnostics = Vec::new();
        diagnostics.extend(self.logs.into_iter().map(|record| {
            format!(
                "host log level={} target={}: {}",
                record.level, record.target, record.message
            )
        }));
        diagnostics.extend(self.diagnostics.into_iter().map(|record| {
            let mut message = format!("host diagnostic {}={}", record.path, record.value);
            if let Some(unit) = record.unit.filter(|unit| !unit.is_empty()) {
                message.push(' ');
                message.push_str(&unit);
            }
            if !record.tags.is_empty() {
                message.push_str(" tags=");
                message.push_str(&record.tags.join(","));
            }
            message
        }));
        diagnostics
    }
}

struct NativePluginHostCallbackCaptureGuard<'a> {
    captures: std::sync::MutexGuard<'a, BTreeMap<u64, NativePluginHostCallbackCapture>>,
    host_handle: u64,
}

impl std::ops::Deref for NativePluginHostCallbackCaptureGuard<'_> {
    type Target = NativePluginHostCallbackCapture;

    fn deref(&self) -> &Self::Target {
        self.captures
            .get(&self.host_handle)
            .expect("native host callback capture should exist while guarded")
    }
}

impl std::ops::DerefMut for NativePluginHostCallbackCaptureGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.captures
            .get_mut(&self.host_handle)
            .expect("native host callback capture should exist while guarded")
    }
}

struct NativePluginHostLogRecord {
    level: u32,
    target: String,
    message: String,
}

struct NativePluginHostDiagnosticRecord {
    path: String,
    value: f64,
    unit: Option<String>,
    tags: Vec<String>,
}

fn native_symbol_name(symbol_name: &str) -> Vec<u8> {
    let mut bytes = symbol_name.as_bytes().to_vec();
    if !bytes.ends_with(&[0]) {
        bytes.push(0);
    }
    bytes
}

fn granted_capabilities_for_entry(
    descriptor: &NativePluginDescriptor,
    module_kind: PluginModuleKind,
) -> Vec<String> {
    let requested = &descriptor.requested_capabilities;
    let mut granted = Vec::new();
    for capability in descriptor
        .package_manifest
        .as_ref()
        .into_iter()
        .flat_map(|manifest| manifest.modules.iter())
        .filter(|module| module.kind == module_kind)
        .flat_map(module_capabilities)
    {
        if requested.iter().any(|requested| requested == capability)
            && !granted.iter().any(|existing| existing == capability)
        {
            granted.push(capability.to_string());
        }
    }
    granted
}

fn module_capabilities(module: &PluginModuleManifest) -> impl Iterator<Item = &str> {
    module.capabilities.iter().map(String::as_str)
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

fn package_manifest_from_toml(
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

fn parse_native_string_list(value: &str) -> Vec<String> {
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
