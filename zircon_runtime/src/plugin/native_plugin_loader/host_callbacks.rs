use std::collections::{BTreeMap, HashSet};
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
use super::ffi_panic_guard::catch_native_plugin_host_callback_panic;
use super::native_plugin_abi::NativePluginDescriptor;
use super::native_strings::{parse_native_string_list, read_optional_c_string};

pub(super) unsafe extern "C" fn native_host_abi_version_v3() -> u32 {
    catch_native_plugin_host_callback_panic(|| ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3)
}

pub(super) unsafe extern "C" fn native_host_abi_version_v2() -> u32 {
    catch_native_plugin_host_callback_panic(|| ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2)
}

pub(super) unsafe extern "C" fn native_host_has_capability_v2(
    host_functions: *const NativePluginHostFunctionTableV2,
    capability: *const std::ffi::c_char,
) -> u32 {
    catch_native_plugin_host_callback_panic(|| unsafe {
        if host_functions.is_null() {
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
        } else {
            native_host_has_capability_from_grants(
                (*host_functions).granted_capabilities,
                capability,
            )
        }
    })
}

pub(super) unsafe extern "C" fn native_host_has_capability_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    capability: *const std::ffi::c_char,
) -> u32 {
    catch_native_plugin_host_callback_panic(|| unsafe {
        native_host_has_capability_v3_inner(host_functions, capability)
    })
}

unsafe fn native_host_has_capability_v3_inner(
    host_functions: *const NativePluginHostFunctionTableV3,
    capability: *const std::ffi::c_char,
) -> u32 {
    if host_functions.is_null() || capability.is_null() {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    }
    native_host_has_capability_from_grants(
        unsafe { (*host_functions).granted_capabilities },
        capability,
    )
}

unsafe fn native_host_has_capability_from_grants(
    granted_capabilities: *const std::ffi::c_char,
    capability: *const std::ffi::c_char,
) -> u32 {
    if capability.is_null() {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    }
    let Some(capability) = unsafe { CStr::from_ptr(capability) }.to_str().ok() else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
    };
    if granted_capabilities.is_null() {
        return ZIRCON_NATIVE_PLUGIN_STATUS_DENIED;
    }
    let Some(granted_capabilities) = (unsafe { CStr::from_ptr(granted_capabilities) })
        .to_str()
        .ok()
    else {
        return ZIRCON_NATIVE_PLUGIN_STATUS_DENIED;
    };
    if native_capability_list_contains(granted_capabilities, capability) {
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    } else {
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED
    }
}

fn native_capability_list_contains(granted_capabilities: &str, capability: &str) -> bool {
    granted_capabilities
        .split(|character| matches!(character, '\n' | ',' | ';'))
        .map(str::trim)
        .filter(|granted_capability| !granted_capability.is_empty())
        .any(|granted_capability| granted_capability == capability)
}

pub(super) unsafe extern "C" fn native_host_log_v3(
    host_functions: *const NativePluginHostFunctionTableV3,
    level: u32,
    target: *const std::ffi::c_char,
    message: *const std::ffi::c_char,
) -> u32 {
    catch_native_plugin_host_callback_panic(|| unsafe {
        native_host_log_v3_inner(host_functions, level, target, message)
    })
}

unsafe fn native_host_log_v3_inner(
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
    catch_native_plugin_host_callback_panic(|| unsafe {
        native_host_diagnostic_v3_inner(host_functions, path, value, unit, tags)
    })
}

unsafe fn native_host_diagnostic_v3_inner(
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
    let requested = descriptor
        .requested_capabilities
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let Some(manifest) = descriptor.package_manifest.as_ref() else {
        return Vec::new();
    };
    let mut granted_capabilities = HashSet::new();
    let mut granted = Vec::new();
    let mut grant_requested = |capability: &str| {
        if requested.contains(capability) && granted_capabilities.insert(capability.to_string()) {
            granted.push(capability.to_string());
        }
    };
    for capability in manifest
        .modules
        .iter()
        .filter(|module| module.kind == module_kind)
        .flat_map(module_capabilities)
    {
        grant_requested(capability);
    }
    for feature in &manifest.feature_extensions {
        let mut has_entry_module = false;
        for module in feature
            .modules
            .iter()
            .filter(|module| module.kind == module_kind)
        {
            has_entry_module = true;
            for capability in module_capabilities(module) {
                grant_requested(capability);
            }
        }
        if has_entry_module {
            for dependency in &feature.dependencies {
                grant_requested(&dependency.capability);
            }
        }
    }
    granted
}

fn module_capabilities(module: &PluginModuleManifest) -> impl Iterator<Item = &str> {
    module.capabilities.iter().map(String::as_str)
}

#[cfg(test)]
mod tests {
    use crate::plugin::{
        PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleKind,
        PluginModuleManifest, PluginPackageManifest,
    };

    use super::{
        granted_capabilities_for_entry, native_capability_list_contains, NativePluginDescriptor,
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    };

    #[test]
    fn native_host_capability_probe_streams_delimited_tokens_without_owned_list_projection() {
        assert!(native_capability_list_contains(
            "runtime.physics, runtime.render;runtime.audio\nruntime.net",
            "runtime.audio"
        ));
        assert!(!native_capability_list_contains(
            "runtime.physics,runtime.rendering",
            "runtime.render"
        ));
        assert!(!native_capability_list_contains(" , ;\n", ""));

        let source = include_str!("host_callbacks.rs")
            .split_once("unsafe fn native_host_has_capability_v3_inner")
            .expect("native host capability callback should exist")
            .1
            .split_once("pub(super) unsafe extern \"C\" fn native_host_log_v3")
            .expect("native host log callback should follow capability callback")
            .0;
        assert!(source.contains("CStr::from_ptr(granted_capabilities)"));
        assert!(source.contains("native_capability_list_contains"));
        assert!(!source.contains("read_optional_c_string"));
        assert!(!source.contains("parse_native_string_list"));

        let grants = include_str!("host_callbacks.rs")
            .split_once("pub(super) fn granted_capabilities_for_entry")
            .expect("entry capability grant projection should exist")
            .1
            .split_once("fn module_capabilities")
            .expect("module capability iterator should follow grant projection")
            .0;
        assert!(grants.contains("collect::<HashSet<_>>()"));
        assert!(grants.contains("requested.contains(capability)"));
        assert!(grants.contains("granted_capabilities.insert(capability.to_string())"));
        assert!(grants.contains("manifest.feature_extensions"));
        assert!(grants.contains("feature.dependencies"));
        assert!(!grants.contains("requested.iter().any"));
        assert!(!grants.contains("granted.iter().any"));
    }

    #[test]
    fn feature_extension_runtime_entry_grants_module_and_dependency_capabilities() {
        let manifest = PluginPackageManifest::new("sound_feature", "Sound Feature")
            .as_feature_extension()
            .with_feature_extension(
                PluginFeatureBundleManifest::new("sound.feature", "Sound Feature", "sound")
                    .with_dependency(PluginFeatureDependency::primary(
                        "sound",
                        "runtime.plugin.sound",
                    ))
                    .with_dependency(PluginFeatureDependency::required(
                        "physics",
                        "runtime.plugin.physics.unrequested",
                    ))
                    .with_runtime_module(
                        PluginModuleManifest::runtime("sound.feature.runtime", "sound_feature")
                            .with_capabilities([
                                "runtime.feature.sound.feature",
                                "runtime.feature.sound.unrequested",
                            ]),
                    ),
            );
        let descriptor = NativePluginDescriptor {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            plugin_id: "sound_feature".to_string(),
            package_manifest: Some(manifest),
            runtime_entry_name: Some("sound_feature_runtime_entry_v3".to_string()),
            editor_entry_name: None,
            requested_capabilities: vec![
                "runtime.plugin.sound".to_string(),
                "runtime.feature.sound.feature".to_string(),
            ],
        };

        assert_eq!(
            granted_capabilities_for_entry(&descriptor, PluginModuleKind::Runtime),
            [
                "runtime.feature.sound.feature".to_string(),
                "runtime.plugin.sound".to_string(),
            ]
        );
        assert!(granted_capabilities_for_entry(&descriptor, PluginModuleKind::Editor).is_empty());
    }
}
