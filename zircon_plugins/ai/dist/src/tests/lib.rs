use std::ffi::CStr;

use zircon_plugin_sdk::native::{
    NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
};

use super::*;

#[test]
fn ai_dist_descriptor_exports_runtime_entry() {
    let descriptor = zircon_native_plugin_descriptor_v3();

    assert!(!descriptor.is_null());
    // SAFETY: 导出函数返回 SDK 静态描述符的地址；已检查非空，描述符在测试期间有效。
    let descriptor = unsafe { &*descriptor };
    assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
    assert_eq!(
        // SAFETY: 此字段指向声明宏生成的静态 NUL 结尾插件 ID，内存在测试期间有效。
        unsafe { CStr::from_ptr(descriptor.plugin_id) },
        CStr::from_bytes_with_nul(NATIVE_PLUGIN_ID).expect("plugin id is nul terminated")
    );
    assert_eq!(
        // SAFETY: 此字段指向声明宏生成的静态 NUL 结尾入口名，内存在测试期间有效。
        unsafe { CStr::from_ptr(descriptor.runtime_entry_name) },
        CStr::from_bytes_with_nul(NATIVE_RUNTIME_ENTRY.cstr())
            .expect("runtime entry is nul terminated")
    );
}

#[test]
fn ai_dist_runtime_entry_reports_registration_manifest() {
    let granted = b"runtime.plugin.ai\nruntime.feature.ai.behavior_tree\nruntime.feature.ai.blackboard\nruntime.feature.ai.perception\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 11,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    let report = zircon_plugin_ai_runtime_entry_v3(&host);

    assert!(!report.is_null());
    // SAFETY: 入口返回 SDK 静态报告的地址；已检查非空，报告在测试期间有效。
    let report = unsafe { &*report };
    assert!(!report.behavior.is_null());
    // SAFETY: 此字段指向 SDK 静态行为表；已检查非空，行为表在测试期间有效。
    let behavior = unsafe { &*report.behavior };
    assert!(!behavior.registration_manifest.is_null());
}

// SAFETY: 宿主按 C ABI 调用此无参回调；函数不访问外部内存，只返回固定 ABI 版本。
unsafe extern "C" fn host_abi_version() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}
