use std::sync::atomic::{AtomicU64, Ordering};

use zircon_plugin_gltf_importer_runtime::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST,
};
use zircon_plugin_sdk::native::{
    self, bytes_from_slice, callback_status as status, owned_bytes, NativePluginByteSliceV2,
    NativePluginCallbackStatusV2, NativePluginOwnedByteBufferV2, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};

const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");

const NEGOTIATED_CAPABILITIES: &[u8] = NATIVE_REQUESTED_CAPABILITIES;
const RUNTIME_DIAGNOSTICS: &[u8] =
    b"gltf_importer dist entry ready; importers remain hosted by the runtime module\0";
const MISSING_HOST_DIAGNOSTICS: &[u8] =
    b"gltf_importer dist entry requires runtime.plugin.gltf_importer host capability\0";
const EMPTY_MANIFEST: &[u8] = b"\0";
const STATE_MAGIC: &[u8; 8] = b"ZRGLTF01";
const STATE_SCHEMA_VERSION: u32 = 1;
const STATE_SAVE_DIAGNOSTICS: &[u8] = b"gltf importer state saved\0";
const STATE_RESTORE_DIAGNOSTICS: &[u8] = b"gltf importer state restored\0";
const STATE_RESTORE_INVALID_DIAGNOSTICS: &[u8] = b"gltf importer state schema invalid\0";
const STATE_OUTPUT_INVALID_DIAGNOSTICS: &[u8] = b"gltf importer state output was null\0";
const UNLOAD_DIAGNOSTICS: &[u8] = b"gltf importer unload completed\0";
static IMPORTER_STATE_EPOCH: AtomicU64 = AtomicU64::new(1);

zircon_plugin_sdk::native_dist_runtime_plugin_v3! {
    plugin_id: NATIVE_PLUGIN_ID,
    package_manifest: PLUGIN_MANIFEST,
    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    runtime_entry: zircon_plugin_gltf_importer_runtime_entry_v3,
    runtime_entry_name: NATIVE_RUNTIME_ENTRY.cstr(),
    requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS,
    runtime: {
        required_capabilities: ["runtime.plugin.gltf_importer"],
        denied_capabilities: [],
        negotiated_capabilities: NEGOTIATED_CAPABILITIES,
        diagnostics: RUNTIME_DIAGNOSTICS,
        is_stateless: false,
        state_schema_version: STATE_SCHEMA_VERSION,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: Some(native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(EMPTY_MANIFEST),
        event_manifest: Some(EMPTY_MANIFEST),
        registration_manifest: Some(NATIVE_RUNTIME_REGISTRATION_MANIFEST),
        invoke_command: None,
        save_state: Some(gltf_importer_save_state),
        restore_state: Some(gltf_importer_restore_state),
        unload: Some(gltf_importer_unload),
        bridge_methods: [],
        on_host_ready: None,
    },
}

unsafe extern "C" fn gltf_importer_save_state(
    output: *mut NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    if output.is_null() {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATE_OUTPUT_INVALID_DIAGNOSTICS,
        );
    }
    let mut bytes = Vec::with_capacity(STATE_MAGIC.len() + std::mem::size_of::<u64>());
    bytes.extend_from_slice(STATE_MAGIC);
    bytes.extend_from_slice(&IMPORTER_STATE_EPOCH.load(Ordering::Acquire).to_le_bytes());
    unsafe { output.write(owned_bytes(bytes)) };
    status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATE_SAVE_DIAGNOSTICS)
}

unsafe extern "C" fn gltf_importer_restore_state(
    state: NativePluginByteSliceV2,
) -> NativePluginCallbackStatusV2 {
    let bytes = unsafe { bytes_from_slice(state) };
    let epoch_offset = STATE_MAGIC.len();
    let Some(epoch_bytes) = bytes
        .get(epoch_offset..)
        .filter(|bytes| bytes.len() == std::mem::size_of::<u64>())
    else {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATE_RESTORE_INVALID_DIAGNOSTICS,
        );
    };
    if !bytes.starts_with(STATE_MAGIC) {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATE_RESTORE_INVALID_DIAGNOSTICS,
        );
    }
    let epoch = u64::from_le_bytes(
        epoch_bytes
            .try_into()
            .expect("state epoch length was checked above"),
    );
    IMPORTER_STATE_EPOCH.store(epoch, Ordering::Release);
    status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATE_RESTORE_DIAGNOSTICS)
}

unsafe extern "C" fn gltf_importer_unload() -> NativePluginCallbackStatusV2 {
    status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, UNLOAD_DIAGNOSTICS)
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use zircon_plugin_sdk::native::{
        NativePluginByteSliceV2, NativePluginHostFunctionTableV3, NativePluginOwnedByteBufferV2,
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
        let mut buffer = NativePluginOwnedByteBufferV2::empty();
        let save = unsafe { gltf_importer_save_state(&mut buffer) };
        assert_eq!(save.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        let state = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) }.to_vec();
        let free = buffer.free.expect("saved state owns a free callback");
        assert_eq!(unsafe { free(buffer) }.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);

        IMPORTER_STATE_EPOCH.store(0, Ordering::Release);
        let restore = unsafe {
            gltf_importer_restore_state(NativePluginByteSliceV2 {
                data: state.as_ptr(),
                len: state.len(),
            })
        };
        assert_eq!(restore.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(IMPORTER_STATE_EPOCH.load(Ordering::Acquire), 37);

        let mut invalid = b"ZRGLTF02".to_vec();
        invalid.extend_from_slice(&91_u64.to_le_bytes());
        let rejected = unsafe {
            gltf_importer_restore_state(NativePluginByteSliceV2 {
                data: invalid.as_ptr(),
                len: invalid.len(),
            })
        };
        assert_eq!(rejected.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(IMPORTER_STATE_EPOCH.load(Ordering::Acquire), 37);

        let missing_output = unsafe { gltf_importer_save_state(std::ptr::null_mut()) };
        assert_eq!(missing_output.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    }

    unsafe extern "C" fn host_abi_version() -> u32 {
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION
    }
}
