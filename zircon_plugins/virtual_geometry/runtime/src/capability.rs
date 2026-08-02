zircon_plugin_sdk::declare_plugin! {
    pub VIRTUAL_GEOMETRY_DECLARATION {
        id: PLUGIN_ID = "virtual_geometry",
        display_name: "Virtual Geometry",
        category: rendering,
        module: MODULE_NAME = "virtual_geometry.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_virtual_geometry_runtime",
        module_description: "Virtualized geometry runtime and render-feature integration",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            VIRTUAL_GEOMETRY_RUNTIME_CAPABILITY = "runtime.plugin.virtual_geometry" => runtime_registration,
            VIRTUAL_GEOMETRY_ADVANCED_RENDER_CAPABILITY = "runtime.render.advanced.virtual_geometry" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_virtual_geometry_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.render.feature", contribution: "plugin.virtual_geometry.render_feature", schema: "zircon.runtime.render-feature/1" },
                    { point: "runtime.render.virtual_geometry_provider", contribution: "plugin.virtual_geometry.runtime", schema: "zircon.runtime.virtual-geometry-provider/1" },
                    { point: "runtime.render.prepare_collector", contribution: "plugin.virtual_geometry.runtime_prepare", schema: "zircon.runtime.render-prepare-collector/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[
    VIRTUAL_GEOMETRY_RUNTIME_CAPABILITY,
    VIRTUAL_GEOMETRY_ADVANCED_RENDER_CAPABILITY,
];
