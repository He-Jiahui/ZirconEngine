zircon_plugin_sdk::declare_plugin! {
    pub SOUND_DECLARATION {
        id: PLUGIN_ID = "sound",
        display_name: "Sound",
        category: runtime,
        module: MODULE_NAME = "sound.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_sound_runtime",
        module_description: "Runtime audio services and sound component integration",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            SOUND_RUNTIME_CAPABILITY = "runtime.plugin.sound" => runtime_registration,
        ],
        maturity: beta,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_sound_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.component_type", contribution: "plugin.sound.components", schema: "zircon.runtime.component-type/1" },
                    { point: "runtime.plugin_option", contribution: "plugin.sound.options", schema: "zircon.runtime.plugin-options/1" },
                    { point: "runtime.plugin_event_catalog", contribution: "plugin.sound.dynamic_events", schema: "zircon.runtime.event-catalog/1" },
                ],
            },
        },
    }
}
pub const SOUND_TIMELINE_ANIMATION_TRACK_CAPABILITY: &str =
    "runtime.feature.sound.timeline_animation_track";
pub const SOUND_RAY_TRACED_CONVOLUTION_REVERB_CAPABILITY: &str =
    "runtime.feature.sound.ray_traced_convolution_reverb";

pub const RUNTIME_CAPABILITIES: &[&str] = &[SOUND_RUNTIME_CAPABILITY];
