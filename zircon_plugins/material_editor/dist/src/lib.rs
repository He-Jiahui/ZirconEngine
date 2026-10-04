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
#[path = "tests/lib.rs"]
mod tests;
