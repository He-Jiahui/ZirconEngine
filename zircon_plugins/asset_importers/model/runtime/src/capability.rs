zircon_plugin_sdk::declare_plugin! {
    pub MODEL_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.model",
        display_name: "Model Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.model.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_model_runtime",
        module_description: "Model asset importer family plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.model" => runtime_registration,
            MESH_IMPORTER_CAPABILITY = "runtime.asset.importer.model.mesh" => runtime_registration,
            CAD_IMPORTER_CAPABILITY = "runtime.asset.importer.model.cad" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_asset_importer_model_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.model",
                    contribution: "plugin.asset_importer.model.runtime",
                    schema: "zircon.runtime.asset-importer.model/1",
                }],
            },
        },
    }
}

pub const IMPORTER_FAMILY: &str = "model";
