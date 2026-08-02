zircon_plugin_sdk::declare_plugin! {
    pub TERRAIN_DECLARATION {
        id: PLUGIN_ID = "terrain",
        display_name: "Terrain",
        category: authoring,
        module: MODULE_NAME = "terrain.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_terrain_runtime",
        module_description: "Terrain heightfield runtime and import services",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            TERRAIN_RUNTIME_CAPABILITY = "runtime.plugin.terrain" => runtime_registration,
        ],
        maturity: beta,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_terrain_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.component_type", contribution: "plugin.terrain.component", schema: "zircon.runtime.component-type/1" },
                    { point: "runtime.asset_importer", contribution: "terrain.heightfield", schema: "zircon.runtime.asset-importer/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[TERRAIN_RUNTIME_CAPABILITY];
