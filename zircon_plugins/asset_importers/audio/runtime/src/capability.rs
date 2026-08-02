zircon_plugin_sdk::declare_plugin! {
    pub AUDIO_ASSET_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "asset_importer.audio",
        display_name: "Audio Asset Importers",
        category: asset_importer,
        module: MODULE_NAME = "asset_importer.audio.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_asset_importer_audio_runtime",
        module_description: "Audio asset importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.audio" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_asset_importer_audio_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.audio",
                    contribution: "plugin.asset_importer.audio.runtime",
                    schema: "zircon.runtime.asset-importer.audio/1",
                }],
            },
        },
    }
}

pub const IMPORTER_FAMILY: &str = "audio";
pub const CODEC_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.audio.codec";
