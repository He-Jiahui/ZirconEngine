zircon_plugin_sdk::declare_plugin! {
    pub GLTF_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "gltf_importer",
        display_name: "glTF Importer",
        category: asset_importer,
        module: MODULE_NAME = "gltf_importer.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_gltf_importer_runtime",
        module_description: "glTF and GLB model importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.gltf_importer",
            IMPORTER_CAPABILITY = "runtime.asset.importer.model.gltf",
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
    }
}
