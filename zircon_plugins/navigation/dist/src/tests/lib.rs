use std::ffi::CStr;

use zircon_plugin_sdk::native::{
    NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
};

use super::*;

#[test]
fn navigation_dist_descriptor_exports_runtime_entry() {
    let descriptor = zircon_native_plugin_descriptor_v3();

    assert!(!descriptor.is_null());
    // SAFETY: 描述符来自本模块的静态导出，前面的非空断言已确认可解引用。
    let descriptor = unsafe { &*descriptor };
    assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
    assert_eq!(
        // SAFETY: plugin_id 指向本模块以 NUL 结尾的静态字节串。
        unsafe { CStr::from_ptr(descriptor.plugin_id) },
        CStr::from_bytes_with_nul(NATIVE_PLUGIN_ID).expect("plugin id is nul terminated")
    );
    assert_eq!(
        // SAFETY: runtime_entry_name 指向本模块以 NUL 结尾的静态入口名字节串。
        unsafe { CStr::from_ptr(descriptor.runtime_entry_name) },
        CStr::from_bytes_with_nul(NATIVE_RUNTIME_ENTRY.cstr())
            .expect("runtime entry is nul terminated")
    );
}

#[test]
fn navigation_dist_runtime_entry_reports_registration_manifest() {
    let granted = b"runtime.plugin.navigation\nruntime.plugin.navigation.recast\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 17,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    let report = zircon_plugin_navigation_runtime_entry_v3(&host);

    assert!(!report.is_null());
    // SAFETY: 报告由本模块入口生成，非空断言后仍保持静态存活。
    let report = unsafe { &*report };
    assert!(!report.behavior.is_null());
    // SAFETY: 报告保持存活且 behavior 已经通过非空断言。
    let behavior = unsafe { &*report.behavior };
    assert!(!behavior.registration_manifest.is_null());
}

unsafe extern "C" fn host_abi_version() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}
