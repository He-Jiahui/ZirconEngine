zircon_plugin_sdk::declare_plugin! {
    pub TEXTURE_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "texture_importer",
        display_name: "Texture Importer",
        category: asset_importer,
        module: MODULE_NAME = "texture_importer.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_texture_importer_runtime",
        module_description: "Texture and image importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.texture_importer" => runtime_registration,
            IMAGE_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.image" => runtime_registration,
            CONTAINER_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.container" => runtime_registration,
            PSD_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.psd" => runtime_registration,
            CUBEMAP_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.cubemap" => runtime_registration,
            ARRAY_IMPORTER_CAPABILITY = "runtime.asset.importer.texture.array" => runtime_registration,
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_texture_importer_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.texture",
                    contribution: "plugin.texture_importer.runtime",
                    schema: "zircon.runtime.asset-importer.texture/1",
                }],
            },
        },
    }
}
