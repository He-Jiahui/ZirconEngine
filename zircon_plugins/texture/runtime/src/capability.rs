zircon_plugin_sdk::declare_plugin! {
    pub TEXTURE_PLUGIN_DECLARATION {
        id: PLUGIN_ID = "texture",
        display_name: "Texture",
        category: runtime,
        module: TEXTURE_RUNTIME_MODULE_NAME = "texture.runtime",
        runtime_crate: TEXTURE_RUNTIME_CRATE_NAME = "zircon_plugin_texture_runtime",
        module_description: "Texture import and runtime metadata plugin",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [TEXTURE_RUNTIME_CAPABILITY = "runtime.plugin.texture"],
        maturity: stable,
        packaging: [source_template, library_embed, native_dynamic],
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[TEXTURE_RUNTIME_CAPABILITY];
