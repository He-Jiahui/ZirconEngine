zircon_plugin_sdk::declare_plugin! {
    pub AUDIO_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.audio",
        display_name: "Audio Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.audio.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_audio_runtime",
        module_description: "Audio asset importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.audio"],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
    }
}

pub const IMPORTER_FAMILY: &str = "audio";
pub const CODEC_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.audio.codec";
