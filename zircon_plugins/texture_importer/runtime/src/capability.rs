zircon_plugin_sdk::declare_plugin! {
    pub TEXTURE_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "texture_importer",
        display_name: "Texture Importer",
        category: asset_importer,
        module: MODULE_NAME = "texture_importer.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_texture_importer_runtime",
        module_description: "Texture and image importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.texture_importer",
            IMAGE_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.image",
            CONTAINER_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.container",
            PSD_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.psd",
            CUBEMAP_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.cubemap",
            ARRAY_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.array",
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
    }
}
