zircon_plugin_sdk::declare_plugin! {
    pub EDITOR_BUILD_EXPORT_DESKTOP_DECLARATION {
        id: PLUGIN_ID = "editor_build_export_desktop",
        display_name: "Desktop Build Export",
        category: platform,
        module: MODULE_NAME = "editor_build_export_desktop.editor",
        crate_name: EDITOR_CRATE_NAME = "zircon_plugin_editor_build_export_desktop_editor",
        module_description: "Desktop export wizard, diagnostics, and packaging reports",
        targets: [editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            CAPABILITY = "editor.extension.build_export_desktop" => editor_registration,
            DIAGNOSTICS_CAPABILITY = "editor.extension.build_export_desktop.diagnostics" => editor_registration,
            NATIVE_DYNAMIC_REPORT_CAPABILITY = "editor.extension.build_export_desktop.native_dynamic_report" => editor_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            editor: {
                entry: NATIVE_EDITOR_ENTRY = "zircon_plugin_editor_build_export_desktop_editor_entry_v3",
                registration_manifest: NATIVE_EDITOR_REGISTRATION_MANIFEST,
                modules: [{ name: "editor", kind: "editor" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

pub const EDITOR_CAPABILITIES: &[&str] = &[
    CAPABILITY,
    DIAGNOSTICS_CAPABILITY,
    NATIVE_DYNAMIC_REPORT_CAPABILITY,
];
