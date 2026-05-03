use zircon_runtime::{
    plugin::PluginDependencyManifest, plugin::PluginEventCatalogManifest,
    plugin::PluginOptionManifest, plugin::PluginPackageManifest,
};

use crate::component::particle_component_descriptors;

pub const PARTICLES_DYNAMIC_EVENT_NAMESPACE: &str = "particles.dynamic_events";

pub fn attach_particles_manifest_contributions(
    manifest: PluginPackageManifest,
) -> PluginPackageManifest {
    particle_component_descriptors().into_iter().fold(
        particle_event_catalogs().into_iter().fold(
            particle_options().into_iter().fold(
                particle_dependencies()
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

pub fn particle_dependencies() -> Vec<PluginDependencyManifest> {
    vec![
        PluginDependencyManifest::new("scene", true).with_capability("runtime.module.scene"),
        PluginDependencyManifest::new("render_graph", true)
            .with_capability("runtime.module.render_graph"),
        PluginDependencyManifest::new("physics", false).with_capability("runtime.plugin.physics"),
        PluginDependencyManifest::new("animation", false)
            .with_capability("runtime.plugin.animation"),
    ]
}

pub fn particle_options() -> Vec<PluginOptionManifest> {
    vec![
        PluginOptionManifest::new("particles.backend", "Particle Backend", "enum", "cpu"),
        PluginOptionManifest::new(
            "particles.max_particles",
            "Max Particles",
            "integer",
            "8192",
        ),
        PluginOptionManifest::new(
            "particles.fixed_preview_dt",
            "Preview Fixed Step",
            "scalar",
            "0.016666667",
        ),
        PluginOptionManifest::new("particles.gpu_fallback", "GPU Fallback", "bool", "true"),
        PluginOptionManifest::new(
            "particles.physics_enabled",
            "Physical Particles",
            "bool",
            "false",
        )
        .with_required_capability("runtime.plugin.physics"),
        PluginOptionManifest::new(
            "particles.animation_control_enabled",
            "Animation Controlled Particles",
            "bool",
            "false",
        )
        .with_required_capability("runtime.plugin.animation"),
    ]
}

pub fn particle_event_catalogs() -> Vec<PluginEventCatalogManifest> {
    vec![PluginEventCatalogManifest::empty(
        PARTICLES_DYNAMIC_EVENT_NAMESPACE,
        1,
    )]
}
