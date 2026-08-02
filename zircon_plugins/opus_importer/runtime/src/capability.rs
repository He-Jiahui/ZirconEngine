zircon_plugin_sdk::declare_plugin! {
    pub OPUS_IMPORTER_DECLARATION {
        id: PLUGIN_ID = "opus_importer",
        display_name: "Opus Audio Importer",
        category: asset_importer,
        module: MODULE_NAME = "opus_importer.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_opus_importer_runtime",
        module_description: "Opus audio importer plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.opus_importer" => runtime_registration,
            OPUS_IMPORTER_CAPABILITY = "runtime.asset.importer.audio.opus" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_opus_importer_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.asset.importer.audio",
                    contribution: "plugin.opus_importer.runtime",
                    schema: "zircon.runtime.asset-importer.audio/1",
                }],
            },
        },
    }
}

pub const OPUS_IMPORTER_ID: &str = "opus_importer.opus";
pub const NATIVE_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.native";
pub const OPUS_IMPORTER_PRIORITY: i32 = 130;
