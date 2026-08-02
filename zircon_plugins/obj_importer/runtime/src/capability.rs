zircon_plugin_sdk::declare_plugin! {
    pub OBJ_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "obj_importer",
        display_name: "OBJ Importer",
        category: asset_importer,
        module: MODULE_NAME = "obj_importer.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_obj_importer_runtime",
        module_description: "Wavefront OBJ model importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.obj_importer" => runtime_registration,
            IMPORTER_CAPABILITY = "runtime.asset.importer.model.obj" => runtime_registration,
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_obj_importer_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.model",
                    contribution: "plugin.obj_importer.runtime",
                    schema: "zircon.runtime.asset-importer.model/1",
                }],
            },
        },
    }
}
