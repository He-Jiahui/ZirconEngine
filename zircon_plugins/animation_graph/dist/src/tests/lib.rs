use std::ffi::CStr;

use zircon_plugin_sdk::native::{
    NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
};

use super::*;

#[test]
fn animation_graph_dist_descriptor_exports_editor_entry_only() {
    let descriptor = zircon_native_plugin_descriptor_v3();

    assert!(!descriptor.is_null());
    let descriptor = unsafe { &*descriptor };
    assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
    assert_eq!(
        unsafe { CStr::from_ptr(descriptor.plugin_id) },
        CStr::from_bytes_with_nul(NATIVE_PLUGIN_ID).expect("plugin id is nul terminated")
    );
    assert!(descriptor.runtime_entry_name.is_null());
    assert_eq!(
        unsafe { CStr::from_ptr(descriptor.editor_entry_name) },
        CStr::from_bytes_with_nul(NATIVE_EDITOR_ENTRY.cstr())
            .expect("editor entry is nul terminated")
    );
}

#[test]
fn animation_graph_dist_editor_entry_reports_behavior() {
    let granted = b"editor.extension.animation_graph_authoring\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 47,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    let report = zircon_plugin_animation_graph_editor_entry_v3(&host);

    assert!(!report.is_null());
    let report = unsafe { &*report };
    assert!(!report.behavior.is_null());
    assert!(!report.bridge_methods.is_null());
    let bridge_methods = unsafe { &*report.bridge_methods };
    assert_eq!(bridge_methods.method_count, 0);
    let behavior = unsafe { &*report.behavior };
    assert!(!behavior.registration_manifest.is_null());
}

unsafe extern "C" fn host_abi_version() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}
