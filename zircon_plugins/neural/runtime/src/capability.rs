zircon_plugin_sdk::declare_plugin! {
    pub NEURAL_DECLARATION {
        id: PLUGIN_ID = "neural",
        display_name: "Neural",
        category: rendering,
        module: MODULE_NAME = "neural.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_neural_runtime",
        module_description: "Neural model assets, CPU reference execution, and compute pass generation",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            NEURAL_RUNTIME_CAPABILITY = "runtime.plugin.neural" => runtime_registration,
            NEURAL_MODEL_ASSET_CAPABILITY = "runtime.asset.neural_model" => runtime_registration,
        ],
        maturity: experimental,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_neural_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] =
    &[NEURAL_RUNTIME_CAPABILITY, NEURAL_MODEL_ASSET_CAPABILITY];

pub const NEURAL_POST_PROCESS_FEATURE_ID: &str = "neural.post_process";
pub const NEURAL_POST_PROCESS_RUNTIME_CAPABILITY: &str = "runtime.feature.neural.post_process";
pub const RENDERING_POST_PROCESS_RUNTIME_CAPABILITY: &str =
    "runtime.feature.rendering.post_process";
