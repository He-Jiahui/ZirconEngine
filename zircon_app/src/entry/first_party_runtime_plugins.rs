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
    let manifest = config.project_plugins.as_ref().cloned().unwrap_or_default();
    first_party_runtime_plugin_registrations_for_manifest_with_render_profile(
        config.target_mode,
        &manifest,
        &config.render_profile,
    )
}

pub(super) fn first_party_runtime_plugin_registrations_for_manifest_with_render_profile(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    render_profile: &RenderProfileBundle,
) -> Vec<RuntimePluginRegistrationReport> {
    let mut registrations =
        first_party_runtime_plugin_registrations_for_manifest(target_mode, manifest);
    let render_profile_overlay =
        render_profile_runtime_plugin_overlay(manifest, target_mode, render_profile);
    if !render_profile_overlay.selections.is_empty() {
        registrations.extend(first_party_runtime_plugin_registrations_for_manifest(
            target_mode,
            &render_profile_overlay,
        ));
    }
    registrations
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

fn render_profile_runtime_plugin_overlay(
    manifest: &ProjectPluginManifest,
    target_mode: RuntimeTargetMode,
    render_profile: &RenderProfileBundle,
) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: runtime_plugins_for_render_profile(render_profile)
            .filter(|runtime_plugin| {
                !manifest
                    .selections
                    .iter()
                    .any(|selection| selection.id == runtime_plugin.key())
            })
            .map(|runtime_plugin| {
                ProjectPluginSelection::runtime_plugin(runtime_plugin, true, false)
                    .with_target_modes([target_mode])
            })
            .collect(),
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

#[cfg(test)]
mod tests {
    use zircon_runtime::core::framework::project::ProjectPluginManifest;

    use super::*;

    #[test]
    fn render_profile_overlay_omits_a_project_selected_runtime_plugin() {
        let manifest = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::HybridGi,
                true,
                false,
            )],
        };
        let render_profile = RenderProfileBundle::default_render()
            .with_features([RenderProductFeature::HybridGlobalIllumination]);

        let overlay = render_profile_runtime_plugin_overlay(
            &manifest,
            RuntimeTargetMode::EditorHost,
            &render_profile,
        );

        assert!(overlay.selections.is_empty());
    }

    #[test]
    fn render_profile_overlay_adds_only_the_missing_runtime_plugin() {
        let render_profile = RenderProfileBundle::default_render()
            .with_features([RenderProductFeature::HybridGlobalIllumination]);

        let overlay = render_profile_runtime_plugin_overlay(
            &ProjectPluginManifest::default(),
            RuntimeTargetMode::EditorHost,
            &render_profile,
        );

        assert_eq!(overlay.selections.len(), 1);
        assert_eq!(overlay.selections[0].id, RuntimePluginId::HybridGi.key());
        assert_eq!(
            overlay.selections[0].target_modes,
            vec![RuntimeTargetMode::EditorHost]
        );
    }
}
