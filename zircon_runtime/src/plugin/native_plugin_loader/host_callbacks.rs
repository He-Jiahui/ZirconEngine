use std::collections::BTreeMap;
use std::ffi::CStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::plugin::{PluginModuleKind, PluginModuleManifest};

use super::abi_declarations::{
    NativePluginHostFunctionTableV2, NativePluginHostFunctionTableV3,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::native_plugin_abi::NativePluginDescriptor;
use super::native_strings::{parse_native_string_list, read_optional_c_string};

pub(super) unsafe extern "C" fn native_host_abi_version_v2() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2
}

pub(super) unsafe extern "C" fn native_host_has_capability_v2(
    host_functions: *const NativePluginHostFunctionTableV2,
    capability: *const std::ffi::c_char,
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

pub(super) unsafe extern "C" fn native_host_abi_version_v3() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3
}

pub(super) unsafe extern "C" fn native_host_has_capability_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capability: *const std::ffi::c_char,
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

pub(super) unsafe extern "C" fn native_host_log_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    level: u32,
    target: *const std::ffi::c_char,
    message: *const std::ffi::c_char,
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

pub(super) unsafe extern "C" fn native_host_diagnostic_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    path: *const std::ffi::c_char,
    value: f64,
    unit: *const std::ffi::c_char,
    tags: *const std::ffi::c_char,
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

pub(super) fn register_native_host_callback_capture() -> u64 {
    static NEXT_HOST_HANDLE: AtomicU64 = AtomicU64::new(2);
    let host_handle = NEXT_HOST_HANDLE.fetch_add(1, Ordering::Relaxed);
    let mut captures = lock_native_host_callback_captures();
    captures.insert(host_handle, NativePluginHostCallbackCapture::default());
    host_handle
}

pub(super) fn take_native_host_callback_diagnostics(host_handle: u64) -> Vec<String> {
    let mut captures = lock_native_host_callback_captures();
    captures
        .remove(&host_handle)
        .unwrap_or_default()
        .into_entry_diagnostics()
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

pub(super) fn granted_capabilities_for_entry(
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
