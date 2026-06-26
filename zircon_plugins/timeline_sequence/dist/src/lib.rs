use zircon_plugin_sdk::native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION;

const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");

const PLUGIN_ID: &[u8] = b"timeline_sequence\0";
const EDITOR_ENTRY: &[u8] = b"zircon_plugin_timeline_sequence_editor_entry_v3\0";
const REQUESTED_CAPABILITIES: &[u8] = b"editor.extension.timeline_sequence_authoring\0";
const NEGOTIATED_CAPABILITIES: &[u8] = b"editor.extension.timeline_sequence_authoring\0";
const EDITOR_DIAGNOSTICS: &[u8] =
    b"timeline_sequence editor dist entry ready; timeline sequence authoring remains hosted by the editor plugin module\0";
const MISSING_HOST_DIAGNOSTICS: &[u8] =
    b"timeline_sequence dist entry requires editor.extension.timeline_sequence_authoring host capability\0";
const EMPTY_MANIFEST: &[u8] = b"\0";

zircon_plugin_sdk::native_dist_editor_plugin_v3! {
    plugin_id: PLUGIN_ID,
    package_manifest: PLUGIN_MANIFEST,
    descriptor_abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    editor_entry: zircon_plugin_timeline_sequence_editor_entry_v3,
    editor_entry_name: EDITOR_ENTRY,
    requested_capabilities: REQUESTED_CAPABILITIES,
    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS,
    editor: {
        required_capabilities: ["editor.extension.timeline_sequence_authoring"],
        denied_capabilities: [],
        negotiated_capabilities: NEGOTIATED_CAPABILITIES,
        diagnostics: EDITOR_DIAGNOSTICS,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: None,
        command_manifest: Some(EMPTY_MANIFEST),
        event_manifest: Some(EMPTY_MANIFEST),
        registration_manifest: None,
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
    fn timeline_sequence_dist_descriptor_exports_editor_entry_only() {
        let descriptor = zircon_native_plugin_descriptor_v3();

        assert!(!descriptor.is_null());
        let descriptor = unsafe { &*descriptor };
        assert_eq!(descriptor.abi_version, ZIRCON_NATIVE_PLUGIN_ABI_VERSION);
        assert_eq!(
            unsafe { CStr::from_ptr(descriptor.plugin_id) },
            CStr::from_bytes_with_nul(PLUGIN_ID).expect("plugin id is nul terminated")
        );
        assert!(descriptor.runtime_entry_name.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(descriptor.editor_entry_name) },
            CStr::from_bytes_with_nul(EDITOR_ENTRY).expect("editor entry is nul terminated")
        );
    }

    #[test]
    fn timeline_sequence_dist_editor_entry_reports_behavior() {
        let granted = b"editor.extension.timeline_sequence_authoring\0";
        let host = NativePluginHostFunctionTableV3 {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
            host_handle: 47,
            granted_capabilities: granted.as_ptr().cast(),
            host_abi_version: Some(host_abi_version),
            host_has_capability: None,
            host_log: None,
            host_diagnostic: None,
        };

        let report = zircon_plugin_timeline_sequence_editor_entry_v3(&host);

        assert!(!report.is_null());
        let report = unsafe { &*report };
        assert!(!report.behavior.is_null());
        assert!(!report.bridge_methods.is_null());
        let bridge_methods = unsafe { &*report.bridge_methods };
        assert_eq!(bridge_methods.method_count, 0);
        let behavior = unsafe { &*report.behavior };
        assert!(behavior.registration_manifest.is_null());
    }

    unsafe extern "C" fn host_abi_version() -> u32 {
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION
    }
}
