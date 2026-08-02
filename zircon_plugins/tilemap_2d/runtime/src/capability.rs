zircon_plugin_sdk::declare_plugin! {
    pub TILEMAP_2D_DECLARATION {
        id: PLUGIN_ID = "tilemap_2d",
        display_name: "Tilemap 2D",
        category: authoring,
        module: MODULE_NAME = "tilemap_2d.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_tilemap_2d_runtime",
        module_description: "2D tilemap runtime and import services",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            TILEMAP_2D_RUNTIME_CAPABILITY = "runtime.plugin.tilemap_2d" => runtime_registration,
        ],
        maturity: beta,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_tilemap_2d_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.component_type", contribution: "plugin.tilemap_2d.component", schema: "zircon.runtime.component-type/1" },
                    { point: "runtime.asset_importer", contribution: "tilemap_2d.tiled", schema: "zircon.runtime.asset-importer/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[TILEMAP_2D_RUNTIME_CAPABILITY];
