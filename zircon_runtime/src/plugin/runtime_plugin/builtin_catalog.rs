use crate::{RuntimePluginId, RuntimeTargetMode};

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn builtin_catalog() -> Vec<Self> {
        [
            (
                "sound",
                "Sound",
                RuntimePluginId::Sound,
                "zircon_plugin_sound_runtime",
                "runtime.plugin.sound",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "texture",
                "Texture",
                RuntimePluginId::Texture,
                "zircon_plugin_texture_runtime",
                "runtime.plugin.texture",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "net",
                "Network",
                RuntimePluginId::Net,
                "zircon_plugin_net_runtime",
                "runtime.plugin.net",
                &[
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::ClientRuntime,
                ][..],
            ),
            (
                "navigation",
                "Navigation",
                RuntimePluginId::Navigation,
                "zircon_plugin_navigation_runtime",
                "runtime.plugin.navigation",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "particles",
                "Particles",
                RuntimePluginId::Particles,
                "zircon_plugin_particles_runtime",
                "runtime.plugin.particles",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "virtual_geometry",
                "Virtual Geometry",
                RuntimePluginId::VirtualGeometry,
                "zircon_plugin_virtual_geometry_runtime",
                "runtime.plugin.virtual_geometry",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
            (
                "hybrid_gi",
                "Hybrid GI",
                RuntimePluginId::HybridGi,
                "zircon_plugin_hybrid_gi_runtime",
                "runtime.plugin.hybrid_gi",
                &[
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ][..],
            ),
        ]
        .into_iter()
        .map(
            |(id, name, runtime_id, crate_name, capability, target_modes)| {
                Self::new(id, name, runtime_id, crate_name)
                    .with_target_modes(target_modes.iter().copied())
                    .with_capability(capability)
            },
        )
        .collect()
    }
}
