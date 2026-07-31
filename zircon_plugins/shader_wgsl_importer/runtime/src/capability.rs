zircon_plugin_sdk::declare_plugin! {
    pub SHADER_WGSL_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "shader_wgsl_importer",
        display_name: "WGSL Shader Importer",
        category: asset_importer,
        module: MODULE_NAME = "shader_wgsl_importer.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_shader_wgsl_importer_runtime",
        module_description: "WGSL shader importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.shader_wgsl_importer",
            IMPORTER_CAPABILITY = "runtime.asset.importer.shader.wgsl",
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
    }
}
