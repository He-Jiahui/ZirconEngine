zircon_plugin_sdk::declare_plugin! {
    pub DATA_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.data",
        display_name: "Data Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.data.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_data_runtime",
        module_description: "Data asset importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.data" => runtime_registration,
            TOML_IMPORTER_CAPABILITY = "runtime.asset.importer.data.toml" => runtime_registration,
            JSON_IMPORTER_CAPABILITY = "runtime.asset.importer.data.json" => runtime_registration,
            YAML_IMPORTER_CAPABILITY = "runtime.asset.importer.data.yaml" => runtime_registration,
            XML_IMPORTER_CAPABILITY = "runtime.asset.importer.data.xml" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_asset_importer_data_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.data",
                    contribution: "plugin.asset_importer.data.runtime",
                    schema: "zircon.runtime.asset-importer.data/1",
                }],
            },
        },
    }
}

pub const IMPORTER_FAMILY: &str = "data";
