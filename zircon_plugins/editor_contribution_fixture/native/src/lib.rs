//! Native editor fixture for the versioned serialized contribution ABI.

use zircon_plugin_sdk::native::{
    self, callback_status, NativePluginByteSliceV2, NativePluginCallbackStatusV2,
    NativePluginOutputSinkV4, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};

zircon_plugin_sdk::declare_plugin! {
    EDITOR_CONTRIBUTION_FIXTURE_DECLARATION {
        id: PLUGIN_ID = "editor_contribution_fixture",
        display_name: "Editor Contribution Fixture",
        category: sdk,
        module: MODULE_NAME = "editor_contribution_fixture.editor",
        crate_name: NATIVE_CRATE_NAME = "zircon_plugin_editor_contribution_fixture_native",
        module_description: "Versioned serialized editor contribution ABI fixture",
        targets: [editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            EDITOR_CAPABILITY = "editor.extension.editor_contribution_fixture" => editor_registration,
        ],
        maturity: experimental,
        packaging: [native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            editor: {
                entry: NATIVE_EDITOR_ENTRY = "zircon_editor_contribution_fixture_entry_v3",
                registration_manifest: NATIVE_EDITOR_REGISTRATION_MANIFEST,
                modules: [{ name: "editor", kind: "editor" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

const PLUGIN_MANIFEST: &str = concat!(include_str!("../../plugin.toml"), "\0");
const EDITOR_DIAGNOSTICS: &[u8] =
    b"editor contribution fixture exposed serialized contribution payload\0";
const MISSING_HOST_DIAGNOSTICS: &[u8] =
    b"editor contribution fixture requires a compatible native editor host\0";
const EDITOR_COMMAND_MANIFEST_TEXT: &str = concat!(
    r#"schema = "zircon.native.command-manifest/4"
commands = []
"#,
    "\0"
);
const EDITOR_COMMAND_MANIFEST: &[u8] = EDITOR_COMMAND_MANIFEST_TEXT.as_bytes();
const EDITOR_CONTRIBUTION_BATCH_SCHEMA: &[u8] = b"zircon.editor.contribution-batch/1\0";
const EDITOR_CONTRIBUTION_BATCH_TEXT: &str = concat!(
    r#"{
  "package_id": "editor_contribution_fixture",
  "contributions": [
    {
      "kind": "view",
      "id": "editor_contribution_fixture.view",
      "schema": "zircon.editor.view/1",
      "title": "Contribution Fixture",
      "category": "SDK"
    },
    {
      "kind": "drawer",
      "id": "editor_contribution_fixture.drawer",
      "schema": "zircon.editor.drawer/1",
      "display_name": "Contribution Fixture"
    },
    {
      "kind": "menu",
      "path": "Tools/Editor Contribution Fixture",
      "schema": "zircon.editor.menu/1",
      "command_id": "editor.contribution_fixture.open"
    },
    {
      "kind": "command",
      "id": "editor.contribution_fixture.open",
      "schema": "zircon.editor.command/1",
      "display_name": "Open Contribution Fixture"
    },
    {
      "kind": "asset_type",
      "id": "editor_contribution_fixture.asset",
      "schema": "zircon.editor.asset-type/1",
      "display_name": "Contribution Fixture Asset",
      "badge": "Fixture",
      "icon_name": "puzzle-piece",
      "color_token": "editor.accent",
      "thumbnail_icon": "puzzle-piece"
    },
    {
      "kind": "settings_page",
      "id": "editor_contribution_fixture.settings",
      "schema": "zircon.editor.settings-page/1",
      "display_name": "Contribution Fixture",
      "category_path": "Plugins/SDK"
    }
  ]
}"#,
    "\0"
);
const EDITOR_CONTRIBUTION_BATCH: &[u8] = EDITOR_CONTRIBUTION_BATCH_TEXT.as_bytes();
const STATUS_COMMAND_DENIED: &[u8] = b"editor contribution fixture has no commands\0";
const STATUS_UNLOADED: &[u8] = b"editor contribution fixture unloaded\0";

zircon_plugin_sdk::native_dist_editor_plugin_v3! {
    plugin_id: NATIVE_PLUGIN_ID,
    package_manifest: PLUGIN_MANIFEST,
    descriptor_abi_version: native::ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
    editor_entry: zircon_editor_contribution_fixture_entry_v3,
    editor_entry_name: NATIVE_EDITOR_ENTRY.cstr(),
    requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
    missing_host_diagnostics: MISSING_HOST_DIAGNOSTICS,
    editor: {
        required_capabilities: ["editor.extension.editor_contribution_fixture"],
        denied_capabilities: [],
        negotiated_capabilities: NATIVE_REQUESTED_CAPABILITIES,
        diagnostics: EDITOR_DIAGNOSTICS,
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: Some(native::NATIVE_COMMAND_MANIFEST_SCHEMA_V4),
        event_manifest_schema: None,
        registration_manifest_schema: Some(EDITOR_CONTRIBUTION_BATCH_SCHEMA),
        command_manifest: Some(EDITOR_COMMAND_MANIFEST),
        event_manifest: None,
        registration_manifest: Some(EDITOR_CONTRIBUTION_BATCH),
        invoke_command: Some(editor_contribution_fixture_invoke_command),
        save_state: None,
        restore_state: None,
        unload: Some(editor_contribution_fixture_unload),
        bridge_methods: [],
        on_host_ready: None,
    },
}

unsafe extern "C" fn editor_contribution_fixture_invoke_command(
    _command_slot: u32,
    _payload: NativePluginByteSliceV2,
    _output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, STATUS_COMMAND_DENIED)
}

unsafe extern "C" fn editor_contribution_fixture_unload() -> NativePluginCallbackStatusV2 {
    callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_UNLOADED)
}
