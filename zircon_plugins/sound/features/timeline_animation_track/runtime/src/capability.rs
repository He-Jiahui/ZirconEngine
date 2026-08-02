pub const FEATURE_ID: &str = "sound.timeline_animation_track";
pub const DIST_PROVIDER_PACKAGE_ID: &str = "sound_timeline_animation_track";
pub const DIST_CRATE_NAME: &str = "zircon_plugin_sound_timeline_animation_dist";
pub const DIST_RUNTIME_ENTRY: &str = "zircon_plugin_sound_timeline_animation_runtime_entry_v3";
pub const EDITOR_CAPABILITY: &str = "editor.feature.sound.timeline_animation_track";

zircon_plugin_sdk::declare_plugin! {
    pub SOUND_TIMELINE_ANIMATION_TRACK_DECLARATION {
        id: PLUGIN_ID = "sound_timeline_animation_track",
        display_name: "Sound Timeline Animation Track Provider",
        category: runtime,
        module: MODULE_NAME = "sound.timeline_animation_track.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_sound_timeline_animation_runtime",
        module_description: "Sound timeline animation track feature provider",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux, macos],
        capabilities: [
            SOUND_RUNTIME_CAPABILITY = "runtime.plugin.sound" => requested_only,
            RUNTIME_CAPABILITY = "runtime.feature.sound.timeline_animation_track" => runtime_registration,
        ],
        maturity: beta,
        packaging: [native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_sound_timeline_animation_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "sound.timeline_animation_track.runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [
                    { point: "runtime.plugin_feature", contribution: "sound.timeline_animation_track", schema: "zircon.runtime.plugin-feature/1" },
                ],
            },
        },
    }
}

pub const RUNTIME_CAPABILITIES: &[&str] = &[RUNTIME_CAPABILITY];
