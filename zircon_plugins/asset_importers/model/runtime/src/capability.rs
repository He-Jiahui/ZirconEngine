zircon_plugin_sdk::declare_plugin! {
    pub MODEL_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.model",
        display_name: "Model Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.model.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_model_runtime",
        module_description: "Model asset importer family plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.model",
            MESH_IMPORTER_CAPABILITY = "runtime.asset.importer.model.mesh",
            CAD_IMPORTER_CAPABILITY = "runtime.asset.importer.model.cad",
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
    }
}

pub const IMPORTER_FAMILY: &str = "model";
