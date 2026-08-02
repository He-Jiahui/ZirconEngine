zircon_plugin_sdk::declare_plugin! {
    pub RUNTIME_DIAGNOSTICS_DECLARATION {
        id: PLUGIN_ID = "runtime_diagnostics",
        display_name: "Runtime Diagnostics",
        category: diagnostics,
        module: MODULE_NAME = "runtime_diagnostics.editor",
        crate_name: EDITOR_CRATE_NAME = "zircon_plugin_runtime_diagnostics_editor",
        module_description: "Embedded runtime diagnostics editor views",
        targets: [editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            CAPABILITY = "editor.extension.runtime_diagnostics" => editor_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            editor: {
                entry: NATIVE_EDITOR_ENTRY = "zircon_plugin_runtime_diagnostics_editor_entry_v3",
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
