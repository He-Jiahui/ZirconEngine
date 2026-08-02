zircon_plugin_sdk::declare_plugin! {
    pub MATERIAL_EDITOR_DECLARATION {
        id: PLUGIN_ID = "material_editor",
        display_name: "Material Editor",
        category: authoring,
        module: MODULE_NAME = "material_editor.editor",
        crate_name: EDITOR_CRATE_NAME = "zircon_plugin_material_editor_editor",
        module_description: "Material graph authoring tools",
        targets: [editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            CAPABILITY = "editor.extension.material_editor_authoring" => editor_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            editor: {
                entry: NATIVE_EDITOR_ENTRY = "zircon_plugin_material_editor_editor_entry_v3",
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
