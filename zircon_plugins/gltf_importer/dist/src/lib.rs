use std::sync::atomic::{AtomicU64, Ordering};

use zircon_plugin_gltf_importer_runtime::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST,
};
use zircon_plugin_sdk::native::{
    self, bytes_from_slice, callback_status as status, owned_bytes, NativePluginByteSliceV3,
    NativePluginCallbackStatusV3, NativePluginOwnedByteBufferV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
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
const STATE_CALLBACK_PANIC_DIAGNOSTICS: &[u8] = b"gltf importer state callback panicked\0";
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
    output: *mut NativePluginOwnedByteBufferV3,
) -> NativePluginCallbackStatusV3 {
    native::catch_native_callback_panic(STATE_CALLBACK_PANIC_DIAGNOSTICS, || unsafe {
        gltf_importer_save_state_inner(output)
    })
}

unsafe fn gltf_importer_save_state_inner(
    output: *mut NativePluginOwnedByteBufferV3,
) -> NativePluginCallbackStatusV3 {
    if output.is_null() {
        return status(
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            STATE_OUTPUT_INVALID_DIAGNOSTICS,
        );
    }
    let mut bytes = Vec::with_capacity(STATE_MAGIC.len() + std::mem::size_of::<u64>());
    bytes.extend_from_slice(STATE_MAGIC);
    bytes.extend_from_slice(&IMPORTER_STATE_EPOCH.load(Ordering::Acquire).to_le_bytes());
    let buffer = match owned_bytes(bytes) {
        Ok(buffer) => buffer,
        Err(error) => return error.status(),
    };
    unsafe { output.write(buffer) };
    status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATE_SAVE_DIAGNOSTICS)
}

unsafe extern "C" fn gltf_importer_restore_state(
    state: NativePluginByteSliceV3,
) -> NativePluginCallbackStatusV3 {
    native::catch_native_callback_panic(STATE_CALLBACK_PANIC_DIAGNOSTICS, || unsafe {
        gltf_importer_restore_state_inner(state)
    })
}

unsafe fn gltf_importer_restore_state_inner(
    state: NativePluginByteSliceV3,
) -> NativePluginCallbackStatusV3 {
    let bytes = unsafe { bytes_from_slice(state) };
    let epoch_offset = STATE_MAGIC.len();
    let Some(encoded_epoch) = bytes
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
    let mut epoch_bytes = [0_u8; std::mem::size_of::<u64>()];
    epoch_bytes.copy_from_slice(encoded_epoch);
    let epoch = u64::from_le_bytes(epoch_bytes);
    IMPORTER_STATE_EPOCH.store(epoch, Ordering::Release);
    status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATE_RESTORE_DIAGNOSTICS)
}

unsafe extern "C" fn gltf_importer_unload() -> NativePluginCallbackStatusV3 {
    native::catch_native_callback_panic(
        STATE_CALLBACK_PANIC_DIAGNOSTICS,
        gltf_importer_unload_inner,
    )
}

fn gltf_importer_unload_inner() -> NativePluginCallbackStatusV3 {
    status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, UNLOAD_DIAGNOSTICS)
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
