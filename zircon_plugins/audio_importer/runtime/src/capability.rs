zircon_plugin_sdk::declare_plugin! {
    pub AUDIO_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "audio_importer",
        display_name: "Audio Importer",
        category: asset_importer,
        module: MODULE_NAME = "audio_importer.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_audio_importer_runtime",
        module_description: "Audio importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.audio_importer" => runtime_registration,
            WAV_IMPORTER_CAPABILITY = "runtime.asset.importer.audio.wav" => runtime_registration,
            CODEC_IMPORTER_CAPABILITY = "runtime.asset.importer.audio.codec" => runtime_registration,
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_audio_importer_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.audio",
                    contribution: "plugin.audio_importer.runtime",
                    schema: "zircon.runtime.asset-importer.audio/1",
                }],
            },
        },
    }
}
