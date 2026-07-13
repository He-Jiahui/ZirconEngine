use zircon_runtime::core::framework::project::{
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use zircon_runtime::core::framework::render::{RenderProductFeature, RenderProfileBundle};
use zircon_runtime::plugin::{RuntimePluginRegistrationReport, RuntimeProfileDescriptor};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::EntryConfig;

pub fn first_party_runtime_plugin_registrations_for_config(
    config: &EntryConfig,
) -> Vec<RuntimePluginRegistrationReport> {
    let mut manifest = config.project_plugin_manifest().unwrap_or_default();
    add_render_profile_runtime_plugin_selections(
        &mut manifest,
        config.target_mode,
        &config.render_profile,
    );
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
    first_party_runtime_plugin_registrations_for_manifest_impl(target_mode, manifest)
}

#[cfg(any(
    feature = "first-party-runtime-plugins",
    feature = "first-party-advanced-render-runtime-plugins",
    feature = "first-party-navigation-runtime-plugin",
    feature = "first-party-zr-vm-language-runtime-plugin"
))]
fn first_party_runtime_plugin_registrations_for_manifest_impl(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<RuntimePluginRegistrationReport> {
    zircon_first_party_runtime_catalog::first_party_runtime_plugin_registrations_for_manifest(
        target_mode,
        manifest,
    )
}

#[cfg(not(any(
    feature = "first-party-runtime-plugins",
    feature = "first-party-advanced-render-runtime-plugins",
    feature = "first-party-navigation-runtime-plugin",
    feature = "first-party-zr-vm-language-runtime-plugin"
)))]
fn first_party_runtime_plugin_registrations_for_manifest_impl(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<RuntimePluginRegistrationReport> {
    let _ = (target_mode, manifest);
    Vec::new()
}

fn add_render_profile_runtime_plugin_selections(
    manifest: &mut ProjectPluginManifest,
    target_mode: RuntimeTargetMode,
    render_profile: &RenderProfileBundle,
) {
    for runtime_plugin in runtime_plugins_for_render_profile(render_profile) {
        if manifest
            .selections
            .iter()
            .any(|selection| selection.id == runtime_plugin.key())
        {
            continue;
        }
        manifest.selections.push(
            ProjectPluginSelection::runtime_plugin(runtime_plugin, true, false)
                .with_target_modes([target_mode]),
        );
    }
}

fn runtime_plugins_for_render_profile(
    render_profile: &RenderProfileBundle,
) -> impl Iterator<Item = RuntimePluginId> + '_ {
    [
        (
            RenderProductFeature::VirtualGeometry,
            RuntimePluginId::VirtualGeometry,
        ),
        (
            RenderProductFeature::HybridGlobalIllumination,
            RuntimePluginId::HybridGi,
        ),
        (RenderProductFeature::Solari, RuntimePluginId::Solari),
    ]
    .into_iter()
    .filter_map(|(feature, runtime_plugin)| {
        render_profile
            .has_feature(feature)
            .then_some(runtime_plugin)
    })
}
