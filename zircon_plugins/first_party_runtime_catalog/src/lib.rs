//! Linked first-party runtime provider catalog.
//!
//! This crate centralizes the optional Rust implementation fan-out for
//! first-party runtime plugins. `zircon_app` projects profiles and manifests,
//! while this catalog maps selected runtime plugin ids to compiled providers.

use zircon_runtime::core::framework::project::{
    resolve_plugin_selections, PluginSelectionResolutionReport, ProjectPluginManifest,
};
use zircon_runtime::plugin::{
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

#[derive(Clone, Debug)]
pub struct FirstPartyRuntimeCatalogReport {
    pub runtime_plugins: PluginSelectionResolutionReport<RuntimePluginRegistrationReport>,
    pub runtime_plugin_features: Vec<RuntimePluginFeatureRegistrationReport>,
}

pub fn first_party_runtime_catalog_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> FirstPartyRuntimeCatalogReport {
    let runtime_plugins =
        first_party_runtime_plugin_registrations_for_manifest(target_mode, manifest);
    let runtime_plugin_features = if runtime_plugins
        .iter()
        .any(|registration| registration.package_manifest.id == RuntimePluginId::Net.key())
    {
        first_party_net_feature_registrations()
    } else {
        Vec::new()
    };

    FirstPartyRuntimeCatalogReport {
        runtime_plugins,
        runtime_plugin_features,
    }
}

#[cfg(feature = "base-runtime-plugins")]
fn first_party_net_feature_registrations() -> Vec<RuntimePluginFeatureRegistrationReport> {
    vec![
        zircon_plugin_net_http_runtime::plugin_feature_registration(),
        zircon_plugin_net_websocket_runtime::plugin_feature_registration(),
    ]
}

#[cfg(not(feature = "base-runtime-plugins"))]
fn first_party_net_feature_registrations() -> Vec<RuntimePluginFeatureRegistrationReport> {
    Vec::new()
}

pub fn first_party_runtime_plugin_registrations_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> PluginSelectionResolutionReport<RuntimePluginRegistrationReport> {
    resolve_plugin_selections(target_mode, manifest, |runtime_id| {
        first_party_registration_for_runtime_plugin(runtime_id.clone())
    })
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
    if _id == RuntimePluginId::Physics {
        return Some(zircon_plugin_physics_runtime::plugin_registration());
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
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::ObjImporter {
        return Some(zircon_plugin_obj_importer_runtime::plugin_registration());
    }
    #[cfg(feature = "base-runtime-plugins")]
    if _id == RuntimePluginId::ShaderWgslImporter {
        return Some(zircon_plugin_shader_wgsl_importer_runtime::plugin_registration());
    }
    #[cfg(feature = "ui-document-importer")]
    if _id == RuntimePluginId::UiDocumentImporter {
        return Some(zircon_plugin_ui_document_importer_runtime::plugin_registration());
    }
    #[cfg(feature = "advanced-render-runtime-plugins")]
    if _id.key() == "neural" {
        return Some(zircon_plugin_neural_runtime::plugin_registration());
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
#[path = "tests/cases.rs"]
mod tests;
