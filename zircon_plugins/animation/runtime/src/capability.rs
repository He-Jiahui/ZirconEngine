zircon_plugin_sdk::declare_plugin! {
    pub ANIMATION_DECLARATION {
        id: PLUGIN_ID = "animation",
        display_name: "Animation",
        category: runtime,
        module: MODULE_NAME = "animation.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_animation_runtime",
        module_description: "Animation scheduling, clip playback, and timeline integration",
        targets: [client_runtime, server_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            ANIMATION_RUNTIME_CAPABILITY = "runtime.plugin.animation" => runtime_registration,
            ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY = "runtime.feature.animation.timeline_event_track" => runtime_registration,
        ],
        maturity: beta,
        packaging: [source_template, library_embed, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_animation_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.animation.evaluation", contribution: "plugin.animation.runtime", schema: "zircon.runtime.animation-evaluation/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[
    ANIMATION_RUNTIME_CAPABILITY,
    ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY,
];
