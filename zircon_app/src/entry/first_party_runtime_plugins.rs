use std::collections::HashSet;

use zircon_runtime::plugin::{
    ProjectPluginManifest, RuntimePluginRegistrationReport, RuntimeProfileDescriptor,
    RuntimeProfileId,
};
use zircon_runtime::{RuntimePluginId, RuntimeTargetMode};

use super::EntryConfig;

pub fn first_party_runtime_plugin_registrations_for_config(
    config: &EntryConfig,
) -> Vec<RuntimePluginRegistrationReport> {
    let Some(manifest) = config.project_plugin_manifest() else {
        return Vec::new();
    };
    first_party_runtime_plugin_registrations_for_manifest(config.target_mode, &manifest)
}

pub fn first_party_runtime_plugin_registrations_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> Vec<RuntimePluginRegistrationReport> {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let manifest = profile.project_manifest();
    first_party_runtime_plugin_registrations_for_manifest(profile.target_mode, &manifest)
}

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

fn first_party_registration_for_runtime_plugin(
    id: RuntimePluginId,
) -> Option<RuntimePluginRegistrationReport> {
    match id {
        #[cfg(feature = "first-party-runtime-plugins")]
        RuntimePluginId::Sound => Some(zircon_plugin_sound_runtime::plugin_registration()),
        #[cfg(feature = "first-party-runtime-plugins")]
        RuntimePluginId::Texture => Some(zircon_plugin_texture_runtime::plugin_registration()),
        #[cfg(feature = "first-party-runtime-plugins")]
        RuntimePluginId::Net => Some(zircon_plugin_net_runtime::plugin_registration()),
        #[cfg(feature = "first-party-navigation-runtime-plugin")]
        RuntimePluginId::Navigation => {
            Some(zircon_plugin_navigation_runtime::plugin_registration())
        }
        #[cfg(feature = "first-party-runtime-plugins")]
        RuntimePluginId::Particles => Some(zircon_plugin_particles_runtime::plugin_registration()),
        #[cfg(feature = "first-party-runtime-plugins")]
        RuntimePluginId::Animation => Some(zircon_plugin_animation_runtime::plugin_registration()),
        #[cfg(feature = "first-party-runtime-plugins")]
        RuntimePluginId::Rendering => Some(zircon_plugin_rendering_runtime::plugin_registration()),
        _ => None,
    }
}
