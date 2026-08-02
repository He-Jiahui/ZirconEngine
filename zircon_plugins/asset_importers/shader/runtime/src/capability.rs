zircon_plugin_sdk::declare_plugin! {
    pub SHADER_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.shader",
        display_name: "Shader Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.shader.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_shader_runtime",
        module_description: "Shader asset importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.shader" => runtime_registration,
            NAGA_IMPORTER_CAPABILITY = "runtime.asset.importer.shader.naga" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_asset_importer_shader_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.shader",
                    contribution: "plugin.asset_importer.shader.runtime",
                    schema: "zircon.runtime.asset-importer.shader/1",
                }],
            },
        },
    }
}

pub const IMPORTER_FAMILY: &str = "shader";
pub const WGSL_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.shader.wgsl";
