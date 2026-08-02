use zircon_plugin_sdk::native::{self, ZIRCON_NATIVE_PLUGIN_ABI_VERSION};
use zircon_plugin_tilemap_2d_runtime::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST,
};

const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");

const RUNTIME_DIAGNOSTICS: &[u8] =
    b"tilemap_2d dist entry ready; tilemap component and importer services remain hosted by the runtime module\0";
const MISSING_HOST_DIAGNOSTICS: &[u8] =
    b"tilemap_2d dist entry requires runtime.plugin.tilemap_2d host capability\0";
const EMPTY_MANIFEST: &[u8] = b"\0";

zircon_plugin_sdk::native_dist_runtime_plugin_v3! {
    plugin_id: NATIVE_PLUGIN_ID,
    package_manifest: PLUGIN_MANIFEST,
    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    runtime_entry: zircon_plugin_tilemap_2d_runtime_entry_v3,
    runtime_entry_name: NATIVE_RUNTIME_ENTRY.cstr(),
    requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS,
    runtime: {
        required_capabilities: ["runtime.plugin.tilemap_2d"],
        denied_capabilities: [],
        negotiated_capabilities: NATIVE_REQUESTED_CAPABILITIES,
        diagnostics: RUNTIME_DIAGNOSTICS,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: Some(native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(EMPTY_MANIFEST),
        event_manifest: Some(EMPTY_MANIFEST),
        registration_manifest: Some(NATIVE_RUNTIME_REGISTRATION_MANIFEST),
        invoke_command: None,
        save_state: None,
        restore_state: None,
        unload: None,
        bridge_methods: [],
        on_host_ready: None,
    },
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use zircon_plugin_sdk::native::{
        NativePluginHostFunctionTableV3, ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    };

    use super::*;

    #[test]
    fn tilemap_2d_dist_descriptor_exports_runtime_entry() {
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
    }

    #[test]
    fn tilemap_2d_dist_runtime_entry_reports_registration_manifest() {
        let granted = b"runtime.plugin.tilemap_2d\0";
        let host = NativePluginHostFunctionTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            host_handle: 43,
            granted_capabilities: granted.as_ptr().cast(),
            host_abi_version: Some(host_abi_version),
            host_has_capability: None,
            host_log: None,
            host_diagnostic: None,
        };

        let report = zircon_plugin_tilemap_2d_runtime_entry_v3(&host);

        assert!(!report.is_null());
        let report = unsafe { &*report };
        assert!(!report.behavior.is_null());
        let behavior = unsafe { &*report.behavior };
        assert!(!behavior.registration_manifest.is_null());
    }

    unsafe extern "C" fn host_abi_version() -> u32 {
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION
    }
}
