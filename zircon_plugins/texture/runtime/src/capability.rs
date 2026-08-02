zircon_plugin_sdk::declare_plugin! {
    pub TEXTURE_PLUGIN_DECLARATION {
        id: PLUGIN_ID = "texture",
        display_name: "Texture",
        category: runtime,
        module: TEXTURE_RUNTIME_MODULE_NAME = "texture.runtime",
        crate_name: TEXTURE_RUNTIME_CRATE_NAME = "zircon_plugin_texture_runtime",
        module_description: "Texture import and runtime metadata plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            TEXTURE_RUNTIME_CAPABILITY = "runtime.plugin.texture" => runtime_registration,
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_texture_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [{
                    point: "runtime.texture.processing",
                    contribution: "plugin.texture.runtime",
                    schema: "zircon.runtime.texture-processing/1",
                }],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[TEXTURE_RUNTIME_CAPABILITY];
