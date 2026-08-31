//! Native editor fixture for the versioned serialized contribution ABI.

use zircon_plugin_sdk::native::{
    self, callback_status, NativePluginByteSliceV3, NativePluginCallbackStatusV3,
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
[[commands]]
name = "editor.contribution_fixture.open"
slot = 0
payload_schema = "zircon.editor.arguments-json/1"
max_output_bytes = 4096
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
      "id": "editor_contribution_fixture.menu.open",
      "schema": "zircon.editor.menu/2",
      "command_id": "editor.contribution_fixture.open",
      "root_id": "tools",
      "root_label_key": "menu.tools.label",
      "group_ids": ["editor_contribution_fixture"],
      "group_label_keys": ["menu.tools.editor_contribution_fixture.label"],
      "leaf_label_key": "command.editor.contribution_fixture.open.label"
    },
    {
      "kind": "command",
      "id": "editor.contribution_fixture.open",
      "schema": "zircon.editor.command/3",
      "localization_bundle_id": "editor_contribution_fixture",
      "label_key": "command.editor.contribution_fixture.open.label",
      "description_key": "command.editor.contribution_fixture.open.description",
      "execution_contract": {
        "result_codec": "zircon.editor.command-result.v1",
        "resource_budget": {
          "max_input_bytes": 65536,
          "max_output_bytes": 4096,
          "max_execution_time_ms": 5000
        }
      }
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
      "kind": "localization_bundle",
      "id": "editor_contribution_fixture",
      "schema": "zircon.editor.localization-bundle/1",
      "locales": {
        "en": {
          "plugin.editor_contribution_fixture.settings.label": "Contribution Fixture",
          "plugin.editor_contribution_fixture.settings.description": "Settings for the editor contribution fixture",
          "plugin.editor_contribution_fixture.category.plugins": "Plugins",
          "plugin.editor_contribution_fixture.category.sdk": "SDK",
          "menu.tools.label": "Tools",
          "menu.tools.editor_contribution_fixture.label": "Editor Contribution Fixture",
          "command.editor.contribution_fixture.open.label": "Open Contribution Fixture",
          "command.editor.contribution_fixture.open.description": "Open the editor contribution fixture"
        },
        "zh-CN": {
          "plugin.editor_contribution_fixture.settings.label": "编辑器贡献夹具",
          "plugin.editor_contribution_fixture.settings.description": "编辑器贡献夹具设置",
          "plugin.editor_contribution_fixture.category.plugins": "插件",
          "plugin.editor_contribution_fixture.category.sdk": "SDK",
          "menu.tools.label": "工具",
          "menu.tools.editor_contribution_fixture.label": "编辑器贡献夹具",
          "command.editor.contribution_fixture.open.label": "打开编辑器贡献夹具",
          "command.editor.contribution_fixture.open.description": "打开编辑器贡献夹具"
        }
      }
    },
    {
      "kind": "settings_page",
      "id": "plugin.editor_contribution_fixture.settings",
      "schema": "zircon.editor.settings-page/2",
      "label_key": "plugin.editor_contribution_fixture.settings.label",
      "description_key": "plugin.editor_contribution_fixture.settings.description",
      "category_keys": [
        "plugin.editor_contribution_fixture.category.plugins",
        "plugin.editor_contribution_fixture.category.sdk"
      ]
    }
  ]
}"#,
    "\0"
);
const EDITOR_CONTRIBUTION_BATCH: &[u8] = EDITOR_CONTRIBUTION_BATCH_TEXT.as_bytes();
const STATUS_COMMAND_COMPLETED: &[u8] = b"editor contribution fixture command completed\0";
const STATUS_COMMAND_DENIED: &[u8] = b"editor contribution fixture command slot is unknown\0";
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
    command_slot: u32,
    _payload: NativePluginByteSliceV3,
    output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV3 {
    if command_slot != 0 {
        return callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, STATUS_COMMAND_DENIED);
    }
    let output_status = unsafe { output.write(br#"{"opened":true}"#) };
    if output_status.code != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        return output_status;
    }
    callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_COMMAND_COMPLETED)
}

unsafe extern "C" fn editor_contribution_fixture_unload() -> NativePluginCallbackStatusV3 {
    callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, STATUS_UNLOADED)
}
