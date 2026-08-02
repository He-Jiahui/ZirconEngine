zircon_plugin_sdk::declare_plugin! {
    pub RENDERING_DECLARATION {
        id: PLUGIN_ID = "rendering",
        display_name: "Rendering",
        category: rendering,
        module: MODULE_NAME = "rendering.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_rendering_runtime",
        module_description: "Rendering feature owner and runtime integration",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            RENDERING_RUNTIME_CAPABILITY = "runtime.plugin.rendering" => runtime_registration,
        ],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_rendering_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[RENDERING_RUNTIME_CAPABILITY];
