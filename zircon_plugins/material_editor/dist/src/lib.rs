use zircon_plugin_material_editor_editor::{
    NATIVE_EDITOR_ENTRY, NATIVE_EDITOR_REGISTRATION_MANIFEST, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES,
};
use zircon_plugin_sdk::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION;

const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");

const EDITOR_DIAGNOSTICS: &[u8] =
    b"material_editor editor dist entry ready; material graph authoring remains hosted by the editor plugin module\0";
const MISSING_HOST_DIAGNOSTICS: &[u8] =
    b"material_editor dist entry requires editor.extension.material_editor_authoring host capability\0";
const EMPTY_MANIFEST: &[u8] = b"\0";

zircon_plugin_sdk::native_dist_editor_plugin_v3! {
    plugin_id: NATIVE_PLUGIN_ID,
    package_manifest: PLUGIN_MANIFEST,
    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    editor_entry: zircon_plugin_material_editor_editor_entry_v3,
    editor_entry_name: NATIVE_EDITOR_ENTRY.cstr(),
    requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS,
    editor: {
        required_capabilities: ["editor.extension.material_editor_authoring"],
        denied_capabilities: [],
        negotiated_capabilities: NATIVE_REQUESTED_CAPABILITIES,
        diagnostics: EDITOR_DIAGNOSTICS,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: Some(zircon_plugin_sdk::native::NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3),
        command_manifest: Some(EMPTY_MANIFEST),
        event_manifest: Some(EMPTY_MANIFEST),
        registration_manifest: Some(NATIVE_EDITOR_REGISTRATION_MANIFEST),
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
    fn material_editor_dist_descriptor_exports_editor_entry_only() {
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
    fn material_editor_dist_editor_entry_reports_behavior() {
        let granted = b"editor.extension.material_editor_authoring\0";
        let host = NativePluginHostFunctionTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            host_handle: 47,
            granted_capabilities: granted.as_ptr().cast(),
            host_abi_version: Some(host_abi_version),
            host_has_capability: None,
            host_log: None,
            host_diagnostic: None,
        };

        let report = zircon_plugin_material_editor_editor_entry_v3(&host);

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
}
