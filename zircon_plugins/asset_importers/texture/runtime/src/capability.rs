zircon_plugin_sdk::declare_plugin! {
    pub TEXTURE_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.texture",
        display_name: "Texture Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.texture.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_texture_runtime",
        module_description: "Texture asset importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.texture"],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
    }
}

pub const IMPORTER_FAMILY: &str = "texture";
pub const RUNTIME_CAPABILITIES: &[&str] = TEXTURE_ASSET_IMPORTER_DECLARATION.capabilities();
pub const CONTAINER_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.container";
pub const PSD_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.psd";
