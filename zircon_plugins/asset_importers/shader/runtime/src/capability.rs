zircon_plugin_sdk::declare_plugin! {
    pub SHADER_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.shader",
        display_name: "Shader Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.shader.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_shader_runtime",
        module_description: "Shader asset importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.shader",
            NAGA_IMPORTER_CAPABILITY = "runtime.asset.importer.shader.naga",
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
    }
}

pub const IMPORTER_FAMILY: &str = "shader";
pub const WGSL_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.shader.wgsl";
