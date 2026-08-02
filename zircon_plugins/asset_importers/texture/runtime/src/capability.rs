zircon_plugin_sdk::declare_plugin! {
    pub TEXTURE_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.texture",
        display_name: "Texture Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.texture.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_texture_runtime",
        module_description: "Texture asset importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.texture" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_asset_importer_texture_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.texture",
                    contribution: "plugin.asset_importer.texture.runtime",
                    schema: "zircon.runtime.asset-importer.texture/1",
                }],
            },
        },
    }
}

pub const IMPORTER_FAMILY: &str = "texture";
pub const RUNTIME_CAPABILITIES: &[&str] = TEXTURE_ASSET_IMPORTER_DECLARATION.capabilities();
pub const CONTAINER_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.container";
pub const PSD_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.texture.psd";
