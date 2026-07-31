zircon_plugin_sdk::declare_plugin! {
    pub DATA_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.data",
        display_name: "Data Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.data.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_data_runtime",
        module_description: "Data asset importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.data",
            TOML_IMPORTER_CAPABILITY = "runtime.asset.importer.data.toml",
            JSON_IMPORTER_CAPABILITY = "runtime.asset.importer.data.json",
            YAML_IMPORTER_CAPABILITY = "runtime.asset.importer.data.yaml",
            XML_IMPORTER_CAPABILITY = "runtime.asset.importer.data.xml",
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
    }
}

pub const IMPORTER_FAMILY: &str = "data";
