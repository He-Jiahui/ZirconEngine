zircon_plugin_sdk::declare_plugin! {
    pub OBJ_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "obj_importer",
        display_name: "OBJ Importer",
        category: asset_importer,
        module: MODULE_NAME = "obj_importer.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_obj_importer_runtime",
        module_description: "Wavefront OBJ model importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.obj_importer",
            IMPORTER_CAPABILITY = "runtime.asset.importer.model.obj",
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
    }
}
