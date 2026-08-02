zircon_plugin_sdk::declare_plugin! {
    pub NATIVE_WINDOW_HOSTING_DECLARATION {
        id: PLUGIN_ID = "native_window_hosting",
        display_name: "Native Window Hosting",
        category: platform,
        module: MODULE_NAME = "native_window_hosting.editor",
        crate_name: EDITOR_CRATE_NAME = "zircon_plugin_native_window_hosting_editor",
        module_description: "Native floating-window editor integration",
        targets: [editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            CAPABILITY = "editor.extension.native_window_hosting" => editor_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            editor: {
                entry: NATIVE_EDITOR_ENTRY = "zircon_plugin_native_window_hosting_editor_entry_v3",
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
