zircon_plugin_sdk::declare_plugin! {
    pub UI_DOCUMENT_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "ui_document_importer",
        display_name: "UI Document Importer",
        category: asset_importer,
        module: MODULE_NAME = "ui_document_importer.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_ui_document_importer_runtime",
        module_description: "UI document importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.ui_document_importer" => runtime_registration,
            IMPORTER_CAPABILITY = "runtime.asset.importer.ui_document" => runtime_registration,
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_ui_document_importer_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.ui_document",
                    contribution: "plugin.ui_document_importer.runtime",
                    schema: "zircon.runtime.asset-importer.ui_document/1",
                }],
            },
        },
    }
}
