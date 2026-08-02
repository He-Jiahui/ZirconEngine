zircon_plugin_sdk::declare_plugin! {
    pub SHADER_WGSL_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "shader_wgsl_importer",
        display_name: "WGSL Shader Importer",
        category: asset_importer,
        module: MODULE_NAME = "shader_wgsl_importer.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_shader_wgsl_importer_runtime",
        module_description: "WGSL shader importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.shader_wgsl_importer" => runtime_registration,
            IMPORTER_CAPABILITY = "runtime.asset.importer.shader.wgsl" => runtime_registration,
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_shader_wgsl_importer_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.shader",
                    contribution: "plugin.shader_wgsl_importer.runtime",
                    schema: "zircon.runtime.asset-importer.shader/1",
                }],
            },
        },
    }
}
