zircon_plugin_sdk::declare_plugin! {
    pub UI_DOCUMENT_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "ui_document_importer",
        display_name: "UI Document Importer",
        category: asset_importer,
        module: MODULE_NAME = "ui_document_importer.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_ui_document_importer_runtime",
        module_description: "UI document importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.ui_document_importer",
            IMPORTER_CAPABILITY = "runtime.asset.importer.ui_document",
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
    }
}
