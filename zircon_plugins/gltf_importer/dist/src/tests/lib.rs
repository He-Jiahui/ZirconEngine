use std::ffi::CStr;

use zircon_plugin_sdk::native::{
    NativePluginByteSliceV3, NativePluginHostFunctionTableV3, NativePluginOwnedByteBufferV3,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};

use super::*;

#[test]
fn gltf_importer_dist_descriptor_exports_runtime_entry() {
    let descriptor = zircon_native_plugin_descriptor_v3();

    assert!(!descriptor.is_null());
    let descriptor = unsafe { &*descriptor };
    assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
    assert_eq!(
        unsafe { CStr::from_ptr(descriptor.plugin_id) },
        CStr::from_bytes_with_nul(NATIVE_PLUGIN_ID).expect("plugin id is nul terminated")
    );
    assert_eq!(
        unsafe { CStr::from_ptr(descriptor.runtime_entry_name) },
        CStr::from_bytes_with_nul(NATIVE_RUNTIME_ENTRY.cstr())
            .expect("runtime entry is nul terminated")
    );
    assert_eq!(
        unsafe { CStr::from_ptr(descriptor.requested_capabilities) },
        CStr::from_bytes_with_nul(NATIVE_REQUESTED_CAPABILITIES)
            .expect("requested capabilities are nul terminated")
    );
}

#[test]
fn gltf_importer_dist_runtime_entry_reports_registration_manifest() {
    let granted = b"runtime.plugin.gltf_importer\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 37,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    let report = zircon_plugin_gltf_importer_runtime_entry_v3(&host);

    assert!(!report.is_null());
    let report = unsafe { &*report };
    assert!(!report.behavior.is_null());
    let behavior = unsafe { &*report.behavior };
    assert_eq!(behavior.is_stateless, 0);
    assert_eq!(
        behavior.schema_versions.state_schema_version,
        STATE_SCHEMA_VERSION
    );
    assert!(behavior.save_state.is_some());
    assert!(behavior.restore_state.is_some());
    assert!(behavior.unload.is_some());
    assert!(!behavior.registration_manifest.is_null());
    assert_eq!(
        unsafe { CStr::from_ptr(report.negotiated_capabilities) },
        CStr::from_bytes_with_nul(NEGOTIATED_CAPABILITIES)
            .expect("negotiated capabilities are nul terminated")
    );
}

#[test]
fn gltf_importer_dist_state_round_trips_and_rejects_another_schema() {
    IMPORTER_STATE_EPOCH.store(37, Ordering::Release);
    let mut buffer = NativePluginOwnedByteBufferV3::empty();
    let save = unsafe { gltf_importer_save_state(&mut buffer) };
    assert_eq!(save.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    let state = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) }.to_vec();
    let free = buffer.free.expect("saved state owns a free callback");
    assert_eq!(unsafe { free(buffer) }.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);

    IMPORTER_STATE_EPOCH.store(0, Ordering::Release);
    let restore = unsafe {
        gltf_importer_restore_state(NativePluginByteSliceV3 {
            data: state.as_ptr(),
            len: state.len(),
        })
    };
    assert_eq!(restore.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(IMPORTER_STATE_EPOCH.load(Ordering::Acquire), 37);

    let mut invalid = b"ZRGLTF02".to_vec();
    invalid.extend_from_slice(&91_u64.to_le_bytes());
    let rejected = unsafe {
        gltf_importer_restore_state(NativePluginByteSliceV3 {
            data: invalid.as_ptr(),
            len: invalid.len(),
        })
    };
    assert_eq!(rejected.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert_eq!(IMPORTER_STATE_EPOCH.load(Ordering::Acquire), 37);

    for invalid_length in [state[..state.len() - 1].to_vec(), {
        let mut bytes = state.clone();
        bytes.push(0);
        bytes
    }] {
        let rejected = unsafe {
            gltf_importer_restore_state(NativePluginByteSliceV3 {
                data: invalid_length.as_ptr(),
                len: invalid_length.len(),
            })
        };
        assert_eq!(rejected.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(IMPORTER_STATE_EPOCH.load(Ordering::Acquire), 37);
    }

    let missing_output = unsafe { gltf_importer_save_state(std::ptr::null_mut()) };
    assert_eq!(missing_output.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
}

unsafe extern "C" fn host_abi_version() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}
