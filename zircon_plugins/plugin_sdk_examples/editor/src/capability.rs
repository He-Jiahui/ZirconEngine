zircon_plugin_sdk::declare_plugin! {
    pub PLUGIN_SDK_EXAMPLES_DECLARATION {
        id: PLUGIN_ID = "plugin_sdk_examples",
        display_name: "Plugin SDK Examples",
        category: sdk,
        module: MODULE_NAME = "plugin_sdk_examples.editor",
        crate_name: EDITOR_CRATE_NAME = "zircon_plugin_sdk_examples_editor",
        module_description: "Editor SDK example extensions and asset fixtures",
        targets: [editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            CAPABILITY = "editor.extension.plugin_sdk_examples" => editor_registration,
            WINDOW_CAPABILITY = "editor.extension.plugin_sdk_examples.window" => editor_registration,
            ASSET_FIXTURE_CAPABILITY = "editor.extension.plugin_sdk_examples.asset_fixture" => editor_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            editor: {
                entry: NATIVE_EDITOR_ENTRY = "zircon_plugin_sdk_examples_editor_entry_v3",
                registration_manifest: NATIVE_EDITOR_REGISTRATION_MANIFEST,
                modules: [{ name: "editor", kind: "editor" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

pub const EDITOR_CAPABILITIES: &[&str] = &[CAPABILITY, WINDOW_CAPABILITY, ASSET_FIXTURE_CAPABILITY];
