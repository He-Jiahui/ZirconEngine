zircon_plugin_sdk::declare_plugin! {
    pub PREFAB_TOOLS_DECLARATION {
        id: PLUGIN_ID = "prefab_tools",
        display_name: "Prefab Tools",
        category: authoring,
        module: MODULE_NAME = "prefab_tools.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_prefab_tools_runtime",
        module_description: "Prefab component, importer, and instancing services",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            PREFAB_TOOLS_RUNTIME_CAPABILITY = "runtime.plugin.prefab_tools" => runtime_registration,
        ],
        maturity: beta,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_prefab_tools_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.component_type", contribution: "plugin.prefab_tools.component", schema: "zircon.runtime.component-type/1" },
                    { point: "runtime.asset_importer", contribution: "prefab_tools.prefab", schema: "zircon.runtime.asset-importer/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[PREFAB_TOOLS_RUNTIME_CAPABILITY];
