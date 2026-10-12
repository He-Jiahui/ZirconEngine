//! 插件入口的 V3 宿主回调：能力查询只读取宿主授权；日志与诊断暂存到本次入口报告。
use std::collections::{BTreeMap, HashSet};
use std::ffi::CStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::plugin::{PluginModuleKind, PluginModuleManifest};

use super::abi_declarations::{
    NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::ffi_panic_guard::catch_native_plugin_host_callback_panic;
use super::native_plugin_abi::NativePluginDescriptor;
use super::native_strings::{parse_native_string_list, read_optional_c_string};

pub(super) unsafe extern "C" fn native_host_abi_version_v3() -> u32 {
    catch_native_plugin_host_callback_panic(|| ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3)
}

/// 查询入口所属模块实际获授的能力；函数表与授权字符串只在同步插件入口期间有效。
///
/// # Safety
///
/// 非空函数表指针必须对齐且指向有效表，非空能力与授权指针必须指向调用期有效的 NUL 结尾字符串。
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
        // SAFETY: 空表已拒绝；入口合约要求此指针保持对齐、可读，授权 CString 由宿主保留到入口返回。
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

/// 把日志复制进本次插件入口的宿主报告；不能把这张入口期表当成运行期日志服务持有。
///
/// # Safety
///
/// 非空表必须在调用期对齐且可读，非空 target/message 必须是有效的 NUL 结尾字符串。
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

// TODO: [CR-PLUGIN-NATIVE-0104] 确认入口期日志与诊断的条数及字节预算；当前暂存 Vec 可持续追加；下一步按既有 R44-P1-44 日志路由议题核实入口资源策略。
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

/// 记录入口期诊断样本，供入口报告展示；unit 和 tags 可缺省。
///
/// # Safety
///
/// 非空表必须在调用期对齐且可读，非空字符串指针须保持可读并有 NUL 终止符。
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

/// 在调用插件入口前创建本次调用的暂存槽，返回只在该入口期间有效的宿主句柄。
pub(super) fn register_native_host_callback_capture() -> u64 {
    static NEXT_HOST_HANDLE: AtomicU64 = AtomicU64::new(2);
    let host_handle = NEXT_HOST_HANDLE.fetch_add(1, Ordering::Relaxed);
    let mut captures = lock_native_host_callback_captures();
    captures.insert(host_handle, NativePluginHostCallbackCapture::default());
    host_handle
}

/// 插件入口返回后移除暂存槽，并把其日志和诊断并入入口报告；晚到的回调无法再写入。
pub(super) fn take_native_host_callback_diagnostics(host_handle: u64) -> Vec<String> {
    let mut captures = lock_native_host_callback_captures();
    captures
        .remove(&host_handle)
        .unwrap_or_default()
        .into_entry_diagnostics()
}

// 从宿主函数表取出本次入口的句柄并持有暂存槽锁，避免同一次入口的回调记录交错。
unsafe fn native_host_callback_capture(
    host_functions: *const NativePluginHostFunctionTableV3,
) -> Option<NativePluginHostCallbackCaptureGuard<'static>> {
    if host_functions.is_null() {
        return None;
    }
    // SAFETY: 空指针已拒绝；ABI 调用方提供入口期仍存活的对齐函数表，此处仅读取宿主创建的句柄。
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
    // 入口报告按日志、诊断两类归并输出；这里保留各类内部顺序，不承诺两类的混合时间顺序。
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

// 整张表的锁随 guard 一起存活，读取到的暂存槽不会在记录追加期间被入口清理端移除。
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

/// 用插件请求与所属模块清单的交集构造授权；能力字符串由宿主创建并交给入口期查询。
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
#[path = "tests/host_callbacks.rs"]
mod tests;
