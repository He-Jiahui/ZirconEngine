zircon_plugin_sdk::declare_plugin! {
    pub OPUS_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "opus_importer",
        display_name: "Opus Audio Importer",
        category: asset_importer,
        module: MODULE_NAME = "opus_importer.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_opus_importer_runtime",
        module_description: "Opus audio importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.opus_importer",
            OPUS_IMPORTER_CAPABILITY = "runtime.asset.importer.audio.opus",
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
    }
}

pub const OPUS_IMPORTER_ID: &str = "opus_importer.opus";
pub const NATIVE_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.native";
pub const OPUS_IMPORTER_PRIORITY: i32 = 130;
