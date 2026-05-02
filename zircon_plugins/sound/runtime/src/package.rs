use zircon_runtime::{
    plugin::PluginDependencyManifest, plugin::PluginEventCatalogManifest, plugin::PluginOptionManifest,
    plugin::PluginPackageManifest,
};

use crate::components::sound_component_descriptors;

pub const SOUND_DYNAMIC_EVENT_NAMESPACE: &str = "sound.dynamic_events";

pub fn attach_sound_manifest_contributions(
    manifest: PluginPackageManifest,
) -> PluginPackageManifest {
    sound_component_descriptors().into_iter().fold(
        sound_event_catalogs().into_iter().fold(
            sound_options().into_iter().fold(
                sound_dependencies()
                    .into_iter()
                    .fold(manifest, |manifest, dependency| {
                        manifest.with_dependency(dependency)
                    }),
                |manifest, option| manifest.with_option(option),
            ),
            |manifest, event_catalog| manifest.with_event_catalog(event_catalog),
        ),
        |manifest, component| manifest.with_component(component),
    )
}

pub fn sound_dependencies() -> Vec<PluginDependencyManifest> {
    vec![
        PluginDependencyManifest::new("asset", true).with_capability("runtime.module.asset"),
        PluginDependencyManifest::new("scene", true).with_capability("runtime.module.scene"),
        PluginDependencyManifest::new("ray_query", false)
            .with_capability("runtime.capability.ray_query"),
        PluginDependencyManifest::new("timeline_sequence", false)
            .with_capability("editor.extension.timeline_authoring"),
    ]
}

pub fn sound_options() -> Vec<PluginOptionManifest> {
    vec![
        PluginOptionManifest::new("sound.backend", "Audio Backend", "string", "software-mixer"),
        PluginOptionManifest::new("sound.sample_rate_hz", "Sample Rate", "integer", "48000"),
        PluginOptionManifest::new("sound.channel_count", "Channel Count", "integer", "2"),
        PluginOptionManifest::new("sound.block_size_frames", "Block Size", "integer", "256"),
        PluginOptionManifest::new("sound.max_voices", "Max Voices", "integer", "128"),
        PluginOptionManifest::new("sound.max_tracks", "Max Tracks", "integer", "64"),
        PluginOptionManifest::new("sound.hrtf_enabled", "HRTF", "bool", "false"),
        PluginOptionManifest::new(
            "sound.convolution_enabled",
            "Convolution Reverb",
            "bool",
            "true",
        ),
        PluginOptionManifest::new(
            "sound.ray_tracing_quality",
            "Ray Tracing Quality",
            "enum",
            "disabled",
        )
        .with_required_capability("runtime.capability.ray_query"),
        PluginOptionManifest::new(
            "sound.timeline_integration",
            "Timeline Automation",
            "bool",
            "true",
        )
        .with_required_capability("editor.extension.timeline_authoring"),
        PluginOptionManifest::new(
            "sound.dynamic_events_enabled",
            "Dynamic Events",
            "bool",
            "true",
        ),
    ]
}

pub fn sound_event_catalogs() -> Vec<PluginEventCatalogManifest> {
    vec![PluginEventCatalogManifest::empty(
        SOUND_DYNAMIC_EVENT_NAMESPACE,
        1,
    )]
}
