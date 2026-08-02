//! Linked first-party runtime provider catalog.
//!
//! This crate centralizes the optional Rust implementation fan-out for
//! first-party runtime plugins. `zircon_app` projects profiles and manifests,
//! while this catalog maps selected runtime plugin ids to compiled providers.

use std::collections::HashSet;

use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

pub fn first_party_runtime_plugin_registrations_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<RuntimePluginRegistrationReport> {
    let mut seen = HashSet::with_capacity(manifest.selections.len());
    let mut registrations = Vec::with_capacity(manifest.selections.len());
    for selection in manifest.enabled_for_target(target_mode) {
        let Some(runtime_id) = RuntimePluginId::parse_key(&selection.id) else {
            continue;
        };
        if !seen.insert(runtime_id.clone()) {
            continue;
        }
        let Some(registration) = first_party_registration_for_runtime_plugin(runtime_id) else {
            continue;
        };
        registrations.push(registration);
    }
    registrations
}

pub fn first_party_registration_for_runtime_plugin(
    _id: RuntimePluginId,
) -> Option<RuntimePluginRegistrationReport> {
    // @cargo-zircon:runtime-registration-begin
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::Ai {
        return Some(zircon_plugin_ai_runtime::plugin_registration());
    }
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::Sound {
        return Some(zircon_plugin_sound_runtime::plugin_registration());
    }
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::Texture {
        return Some(zircon_plugin_texture_runtime::plugin_registration());
    }
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::Net {
        return Some(zircon_plugin_net_runtime::plugin_registration());
    }
    #[cfg(feature = "navigation-runtime-plugin")]
    if _id == RuntimePluginId::Navigation {
        return Some(zircon_plugin_navigation_runtime::plugin_registration());
    }
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::Particles {
        return Some(zircon_plugin_particles_runtime::plugin_registration());
    }
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::Animation {
        return Some(zircon_plugin_animation_runtime::plugin_registration());
    }
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::Rendering {
        return Some(zircon_plugin_rendering_runtime::plugin_registration());
    }
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::GltfImporter {
        return Some(zircon_plugin_gltf_importer_runtime::plugin_registration());
    }
    #[cfg(feature = "advanced-render-runtime-plugins")]
    if _id == RuntimePluginId::VirtualGeometry {
        return Some(zircon_plugin_virtual_geometry_runtime::plugin_registration());
    }
    #[cfg(feature = "advanced-render-runtime-plugins")]
    if _id == RuntimePluginId::HybridGi {
        return Some(zircon_plugin_hybrid_gi_runtime::plugin_registration());
    }
    #[cfg(feature = "advanced-render-runtime-plugins")]
    if _id == RuntimePluginId::Solari {
        return Some(zircon_plugin_solari_runtime::plugin_registration());
    }
    #[cfg(feature = "zr-vm-language-runtime-plugin")]
    if _id == RuntimePluginId::ZrVmLanguage {
        return Some(zircon_plugin_zr_vm_language_runtime::plugin_registration());
    }
    // @cargo-zircon:runtime-registration-end
    None
}

#[cfg(test)]
mod tests;
