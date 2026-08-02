zircon_plugin_sdk::declare_plugin! {
    pub UI_ASSET_AUTHORING_DECLARATION {
        id: PLUGIN_ID = "ui_asset_authoring",
        display_name: "UI Asset Authoring",
        category: authoring,
        module: MODULE_NAME = "ui_asset_authoring.editor",
        crate_name: EDITOR_CRATE_NAME = "zircon_plugin_ui_asset_authoring_editor",
        module_description: "Retained UI asset authoring editor tools",
        targets: [editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            CAPABILITY = "editor.extension.ui_asset_authoring" => editor_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            editor: {
                entry: NATIVE_EDITOR_ENTRY = "zircon_plugin_ui_asset_authoring_editor_entry_v3",
                registration_manifest: NATIVE_EDITOR_REGISTRATION_MANIFEST,
                modules: [{ name: "editor", kind: "editor" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

pub const EDITOR_CAPABILITIES: &[&str] = &[CAPABILITY];
