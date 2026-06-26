use zircon_plugin_sdk::native::{self, ZIRCON_NATIVE_PLUGIN_ABI_VERSION};

const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");

const PLUGIN_ID: &[u8] = b"texture_importer\0";
const RUNTIME_ENTRY: &[u8] = b"zircon_plugin_texture_importer_runtime_entry_v3\0";
const REQUESTED_CAPABILITIES: &[u8] = b"runtime.plugin.texture_importer\nruntime.asset.importer.texture.image\nruntime.asset.importer.texture.container\nruntime.asset.importer.texture.psd\0";
const NEGOTIATED_CAPABILITIES: &[u8] = REQUESTED_CAPABILITIES;
const RUNTIME_DIAGNOSTICS: &[u8] =
    b"texture_importer dist entry ready; importers remain hosted by the runtime module\0";
const MISSING_HOST_DIAGNOSTICS: &[u8] =
    b"texture_importer dist entry requires runtime.plugin.texture_importer host capability\0";
const EMPTY_MANIFEST: &[u8] = b"\0";
const RUNTIME_REGISTRATION_MANIFEST: &[u8] = b"schema = \"zircon.native.registration-manifest/3\"\ncapabilities = [\"runtime.plugin.texture_importer\", \"runtime.asset.importer.texture.image\", \"runtime.asset.importer.texture.container\", \"runtime.asset.importer.texture.psd\"]\n[[modules]]\nname = \"runtime\"\nkind = \"runtime\"\n[[extensions]]\npoint = \"runtime.asset.importer.texture\"\ncontribution = \"plugin.texture_importer.runtime\"\nschema = \"zircon.runtime.asset-importer.texture/1\"\n\0";

zircon_plugin_sdk::native_dist_runtime_plugin_v3! {
    plugin_id: PLUGIN_ID,
    package_manifest: PLUGIN_MANIFEST,
    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    runtime_entry: zircon_plugin_texture_importer_runtime_entry_v3,
    runtime_entry_name: RUNTIME_ENTRY,
    requested_capabilities: REQUESTED_CAPABILITIES,
    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS,
    runtime: {
        required_capabilities: ["runtime.plugin.texture_importer"],
        denied_capabilities: [],
        negotiated_capabilities: NEGOTIATED_CAPABILITIES,
        diagnostics: RUNTIME_DIAGNOSTICS,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: Some(native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(EMPTY_MANIFEST),
        event_manifest: Some(EMPTY_MANIFEST),
        registration_manifest: Some(RUNTIME_REGISTRATION_MANIFEST),
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
    fn texture_importer_dist_descriptor_exports_runtime_entry() {
        let descriptor = zircon_native_plugin_descriptor_v3();

        assert!(!descriptor.is_null());
        let descriptor = unsafe { &*descriptor };
        assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
        assert_eq!(
            unsafe { CStr::from_ptr(descriptor.plugin_id) },
            CStr::from_bytes_with_nul(PLUGIN_ID).expect("plugin id is nul terminated")
        );
        assert_eq!(
            unsafe { CStr::from_ptr(descriptor.runtime_entry_name) },
            CStr::from_bytes_with_nul(RUNTIME_ENTRY).expect("runtime entry is nul terminated")
        );
        assert_eq!(
            unsafe { CStr::from_ptr(descriptor.requested_capabilities) },
            CStr::from_bytes_with_nul(REQUESTED_CAPABILITIES)
                .expect("requested capabilities are nul terminated")
        );
    }

    #[test]
    fn texture_importer_dist_runtime_entry_reports_registration_manifest() {
        let granted = b"runtime.plugin.texture_importer\0";
        let host = NativePluginHostFunctionTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            host_handle: 41,
            granted_capabilities: granted.as_ptr().cast(),
            host_abi_version: Some(host_abi_version),
            host_has_capability: None,
            host_log: None,
            host_diagnostic: None,
        };

        let report = zircon_plugin_texture_importer_runtime_entry_v3(&host);

        assert!(!report.is_null());
        let report = unsafe { &*report };
        assert!(!report.behavior.is_null());
        let behavior = unsafe { &*report.behavior };
        assert!(!behavior.registration_manifest.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(report.negotiated_capabilities) },
            CStr::from_bytes_with_nul(NEGOTIATED_CAPABILITIES)
                .expect("negotiated capabilities are nul terminated")
        );
    }

    unsafe extern "C" fn host_abi_version() -> u32 {
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION
    }
}
