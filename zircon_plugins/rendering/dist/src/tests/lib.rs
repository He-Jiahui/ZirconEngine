use std::ffi::CStr;

use zircon_plugin_sdk::native::{
    NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
};

use super::*;

#[test]
fn rendering_dist_descriptor_exports_runtime_entry() {
    let descriptor = zircon_native_plugin_descriptor_v3();

    assert!(!descriptor.is_null());
    // SAFETY: 当前库的导出宏返回本库静态描述符，前一断言已排除空指针；测试期间库不会卸载。
    let descriptor = unsafe { &*descriptor };
    assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
    assert_eq!(
        // SAFETY: 宏把此字段直接指向主包的静态插件 ID 字节串，该常量末尾保留 NUL。
        unsafe { CStr::from_ptr(descriptor.plugin_id) },
        CStr::from_bytes_with_nul(NATIVE_PLUGIN_ID).expect("plugin id is nul terminated")
    );
    assert_eq!(
        // SAFETY: 宏借用固定入口身份的静态 C 字符串，cstr() 提供结尾 NUL 且测试不修改它。
        unsafe { CStr::from_ptr(descriptor.runtime_entry_name) },
        CStr::from_bytes_with_nul(NATIVE_RUNTIME_ENTRY.cstr())
            .expect("runtime entry is nul terminated")
    );
}

#[test]
fn rendering_dist_runtime_entry_reports_registration_manifest() {
    let granted = b"runtime.plugin.rendering\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 43,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    let report = zircon_plugin_rendering_runtime_entry_v3(&host);

    assert!(!report.is_null());
    // SAFETY: 同库入口按有效宿主表返回静态报告；已核对非空，局部宿主表及库在读取期间仍存活。
    let report = unsafe { &*report };
    assert!(!report.behavior.is_null());
    // SAFETY: 成功报告指向宏生成的静态行为表，前一断言确认此处不是缺少宿主能力的空行为报告。
    let behavior = unsafe { &*report.behavior };
    assert!(!behavior.registration_manifest.is_null());
}

// SAFETY: 这是供宿主函数表调用的 C ABI 回调，只返回版本常量，不解引用外部内存。
unsafe extern "C" fn host_abi_version() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}
