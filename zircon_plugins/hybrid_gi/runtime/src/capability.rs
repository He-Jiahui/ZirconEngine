zircon_plugin_sdk::declare_plugin! {
    pub HYBRID_GI_DECLARATION {
        id: PLUGIN_ID = "hybrid_gi",
        display_name: "Hybrid GI",
        category: rendering,
        module: MODULE_NAME = "hybrid_gi.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_hybrid_gi_runtime",
        module_description: "Hybrid global illumination runtime and render-feature integration",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            HYBRID_GI_RUNTIME_CAPABILITY = "runtime.plugin.hybrid_gi" => runtime_registration,
            HYBRID_GI_ADVANCED_RENDER_CAPABILITY = "runtime.render.advanced.hybrid_gi" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_hybrid_gi_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.render.feature", contribution: "plugin.hybrid_gi.render_feature", schema: "zircon.runtime.render-feature/1" },
                    { point: "runtime.render.hybrid_gi_provider", contribution: "plugin.hybrid_gi.runtime", schema: "zircon.runtime.hybrid-gi-provider/1" },
                    { point: "runtime.render.prepare_collector", contribution: "plugin.hybrid_gi.runtime_prepare", schema: "zircon.runtime.render-prepare-collector/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[
    HYBRID_GI_RUNTIME_CAPABILITY,
    HYBRID_GI_ADVANCED_RENDER_CAPABILITY,
];
