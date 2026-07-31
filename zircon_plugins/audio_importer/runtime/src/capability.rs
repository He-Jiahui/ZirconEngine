zircon_plugin_sdk::declare_plugin! {
    pub AUDIO_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "audio_importer",
        display_name: "Audio Importer",
        category: asset_importer,
        module: MODULE_NAME = "audio_importer.runtime",
        runtime_crate: RUNTIME_CRATE_NAME = "zircon_plugin_audio_importer_runtime",
        module_description: "Audio importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.audio_importer",
            WAV_IMPORTER_CAPABILITY = "runtime.asset.importer.audio.wav",
            CODEC_IMPORTER_CAPABILITY = "runtime.asset.importer.audio.codec",
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
    }
}
