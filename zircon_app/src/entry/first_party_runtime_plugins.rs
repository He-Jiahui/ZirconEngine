use zircon_runtime::core::framework::project::{ProjectPluginManifest, RuntimeProfileId};
use zircon_runtime::core::framework::render::RenderProfileBundle;
use zircon_runtime::plugin::{RuntimePluginRegistrationReport, RuntimeProfileDescriptor};
use zircon_runtime::{
    builtin::{manifest_with_mode_baseline, RuntimePluginId},
    core::framework::platform::RuntimeTargetMode,
};

use super::{
    builtin_modules::{
        effective_project_plugin_manifest, effective_project_plugin_manifest_with_render_profile,
    },
    ResolvedProductHostConfig,
};

pub fn first_party_runtime_plugin_registrations_for_config(
    config: &ResolvedProductHostConfig,
) -> Vec<RuntimePluginRegistrationReport> {
    let effective_manifest = effective_project_plugin_manifest(config);
    first_party_runtime_plugin_registrations_for_manifest_impl(
        config.target_mode(),
        &effective_manifest,
    )
}

pub(super) fn first_party_runtime_plugin_registrations_for_manifest_with_render_profile(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    render_profile: &RenderProfileBundle,
) -> Vec<RuntimePluginRegistrationReport> {
    let effective_manifest = effective_project_plugin_manifest_with_render_profile(
        target_mode,
        Some(manifest),
        render_profile,
    );
    first_party_runtime_plugin_registrations_for_manifest_impl(target_mode, &effective_manifest)
}

pub fn first_party_runtime_plugin_registrations_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> Vec<RuntimePluginRegistrationReport> {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let profile_manifest = profile.project_manifest();
    let manifest = manifest_with_mode_baseline(profile.target_mode, Some(&profile_manifest));
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
    feature = "first-party-ui-document-importer",
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
    feature = "first-party-ui-document-importer",
    feature = "first-party-zr-vm-language-runtime-plugin"
)))]
fn first_party_runtime_plugin_registrations_for_manifest_impl(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<RuntimePluginRegistrationReport> {
    let _ = (target_mode, manifest);
    Vec::new()
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
    use zircon_runtime::core::framework::render::RenderProductFeature;

    use super::*;
    use crate::entry::builtin_modules::render_profile_runtime_plugin_overlay;

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

    #[cfg(feature = "first-party-ui-document-importer")]
    #[test]
    fn ui_product_baseline_projects_one_document_importer_provider() {
        let config = crate::entry::EntryConfig::new(crate::entry::EntryProfile::Runtime)
            .resolve()
            .expect("runtime product host config should resolve");
        let registrations = first_party_runtime_plugin_registrations_for_config(&config);

        assert_eq!(
            registrations
                .iter()
                .filter(|registration| {
                    registration.package_manifest.id == RuntimePluginId::UiDocumentImporter.key()
                })
                .count(),
            1
        );
    }

    #[cfg(feature = "first-party-ui-document-importer")]
    #[test]
    fn project_manifest_can_disable_the_linked_document_importer_provider() {
        let config = crate::entry::EntryConfig::new(crate::entry::EntryProfile::Runtime)
            .with_project_plugins(ProjectPluginManifest {
                selections: vec![ProjectPluginSelection::runtime_plugin(
                    RuntimePluginId::UiDocumentImporter,
                    false,
                    false,
                )],
            })
            .resolve()
            .expect("runtime product host config should resolve");
        let registrations = first_party_runtime_plugin_registrations_for_config(&config);

        assert!(registrations.iter().all(|registration| {
            registration.package_manifest.id != RuntimePluginId::UiDocumentImporter.key()
        }));
    }
}
