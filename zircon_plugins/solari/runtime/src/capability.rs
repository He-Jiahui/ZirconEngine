zircon_plugin_sdk::declare_plugin! {
    pub SOLARI_DECLARATION {
        id: PLUGIN_ID = "solari",
        display_name: "Solari",
        category: rendering,
        module: MODULE_NAME = "solari.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_solari_runtime",
        module_description: "Experimental realtime raytraced lighting provider",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.solari" => runtime_registration,
            SOLARI_CAPABILITY = "runtime.render.experimental.solari" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_solari_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.render.solari_provider", contribution: "plugin.solari.runtime", schema: "zircon.runtime.solari-provider/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[RUNTIME_CAPABILITY, SOLARI_CAPABILITY];
