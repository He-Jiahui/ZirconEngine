//! Linked first-party runtime provider catalog.
//!
//! This crate centralizes the optional Rust implementation fan-out for
//! first-party runtime plugins. `zircon_app` projects profiles and manifests,
//! while this catalog maps selected runtime plugin ids to compiled providers.

use std::collections::HashSet;

use zircon_runtime::plugin::{ProjectPluginManifest, RuntimePluginRegistrationReport};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};

pub fn first_party_runtime_plugin_registrations_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<RuntimePluginRegistrationReport> {
    let mut seen = HashSet::new();
    manifest
        .enabled_for_target(target_mode)
        .filter_map(|selection| selection.runtime_id())
        .filter(|runtime_id| seen.insert(*runtime_id))
        .filter_map(first_party_registration_for_runtime_plugin)
        .collect()
}

pub fn first_party_registration_for_runtime_plugin(
    id: RuntimePluginId,
) -> Option<RuntimePluginRegistrationReport> {
    match id {
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Ai => Some(zircon_plugin_ai_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Sound => Some(zircon_plugin_sound_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Texture => Some(zircon_plugin_texture_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Net => Some(zircon_plugin_net_runtime::plugin_registration()),
        #[cfg(feature = "navigation-runtime-plugin")]
        RuntimePluginId::Navigation => {
            Some(zircon_plugin_navigation_runtime::plugin_registration())
        }
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Particles => Some(zircon_plugin_particles_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Animation => Some(zircon_plugin_animation_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Rendering => Some(zircon_plugin_rendering_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::GltfImporter => {
            Some(zircon_plugin_gltf_importer_runtime::plugin_registration())
        }
        #[cfg(feature = "advanced-render-runtime-plugins")]
        RuntimePluginId::VirtualGeometry => {
            Some(zircon_plugin_virtual_geometry_runtime::plugin_registration())
        }
        #[cfg(feature = "advanced-render-runtime-plugins")]
        RuntimePluginId::HybridGi => Some(zircon_plugin_hybrid_gi_runtime::plugin_registration()),
        #[cfg(feature = "advanced-render-runtime-plugins")]
        RuntimePluginId::Solari => Some(zircon_plugin_solari_runtime::plugin_registration()),
        #[cfg(feature = "zr-vm-language-runtime-plugin")]
        RuntimePluginId::ZrVmLanguage => {
            Some(zircon_plugin_zr_vm_language_runtime::plugin_registration())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::plugin::{ProjectPluginManifest, ProjectPluginSelection};

    use super::*;

    #[cfg(not(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    )))]
    #[test]
    fn catalog_without_provider_features_returns_no_registrations() {
        let manifest = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Sound,
                true,
                true,
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])],
        };

        assert!(first_party_runtime_plugin_registrations_for_manifest(
            RuntimeTargetMode::ClientRuntime,
            &manifest
        )
        .is_empty());
    }
}
