pub const FEATURE_ID: &str = "sound.ray_traced_convolution_reverb";
pub const DIST_PROVIDER_PACKAGE_ID: &str = "sound_ray_traced_convolution_reverb";
pub const DIST_CRATE_NAME: &str = "zircon_plugin_sound_ray_traced_convolution_dist";
pub const DIST_RUNTIME_ENTRY: &str = "zircon_plugin_sound_ray_traced_convolution_runtime_entry_v3";
pub const EDITOR_CAPABILITY: &str = "editor.feature.sound.ray_traced_convolution_reverb";

zircon_plugin_sdk::declare_plugin! {
    pub SOUND_RAY_TRACED_CONVOLUTION_REVERB_DECLARATION {
        id: PLUGIN_ID = "sound_ray_traced_convolution_reverb",
        display_name: "Sound Ray Traced Convolution Reverb Provider",
        category: runtime,
        module: MODULE_NAME = "sound.ray_traced_convolution_reverb.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_sound_ray_traced_convolution_runtime",
        module_description: "Ray-traced convolution reverb feature provider",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            SOUND_RUNTIME_CAPABILITY = "runtime.plugin.sound" => requested_only,
            RUNTIME_CAPABILITY = "runtime.feature.sound.ray_traced_convolution_reverb" => runtime_registration,
        ],
        maturity: beta,
        packaging: [native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_sound_ray_traced_convolution_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "sound.ray_traced_convolution_reverb.runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.plugin_feature", contribution: "sound.ray_traced_convolution_reverb", schema: "zircon.runtime.plugin-feature/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[RUNTIME_CAPABILITY];
