zircon_plugin_sdk::declare_plugin! {
    pub GLTF_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "gltf_importer",
        display_name: "glTF Importer",
        category: asset_importer,
        module: MODULE_NAME = "gltf_importer.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_gltf_importer_runtime",
        module_description: "glTF and GLB model importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.gltf_importer" => runtime_registration,
            IMPORTER_CAPABILITY = "runtime.asset.importer.model.gltf" => runtime_registration,
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_gltf_importer_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.model",
                    contribution: "plugin.gltf_importer.runtime",
                    schema: "zircon.runtime.asset-importer.model/1",
                }],
            },
        },
    }
}
