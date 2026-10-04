use zircon_runtime::core::framework::project::{
    resolve_plugin_selections, PluginSelectionResolutionReport, ProjectPluginManifest,
    RuntimeProfileId,
};
use zircon_runtime::core::framework::render::RenderProfileBundle;
use zircon_runtime::plugin::{
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
    RuntimeProfileDescriptor,
};
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

pub(super) struct FirstPartyRuntimeCatalogReports {
    pub runtime_plugins: PluginSelectionResolutionReport<RuntimePluginRegistrationReport>,
    pub runtime_plugin_features: Vec<RuntimePluginFeatureRegistrationReport>,
}

pub(super) fn first_party_runtime_catalog_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> FirstPartyRuntimeCatalogReports {
    #[cfg(any(
        feature = "first-party-runtime-plugins",
        feature = "first-party-advanced-render-runtime-plugins",
        feature = "first-party-navigation-runtime-plugin",
        feature = "first-party-ui-document-importer",
        feature = "first-party-zr-vm-language-runtime-plugin"
    ))]
    {
        let report = zircon_first_party_runtime_catalog::first_party_runtime_catalog_for_manifest(
            target_mode,
            manifest,
        );
        return FirstPartyRuntimeCatalogReports {
            runtime_plugins: report.runtime_plugins,
            runtime_plugin_features: report.runtime_plugin_features,
        };
    }

    #[cfg(not(any(
        feature = "first-party-runtime-plugins",
        feature = "first-party-advanced-render-runtime-plugins",
        feature = "first-party-navigation-runtime-plugin",
        feature = "first-party-ui-document-importer",
        feature = "first-party-zr-vm-language-runtime-plugin"
    )))]
    {
        FirstPartyRuntimeCatalogReports {
            runtime_plugins: resolve_plugin_selections(target_mode, manifest, |_| None),
            runtime_plugin_features: Vec::new(),
        }
    }
}

pub fn first_party_runtime_plugin_registrations_for_config(
    config: &ResolvedProductHostConfig,
) -> PluginSelectionResolutionReport<RuntimePluginRegistrationReport> {
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
) -> PluginSelectionResolutionReport<RuntimePluginRegistrationReport> {
    let effective_manifest = effective_project_plugin_manifest_with_render_profile(
        target_mode,
        Some(manifest),
        render_profile,
    );
    first_party_runtime_plugin_registrations_for_manifest_impl(target_mode, &effective_manifest)
}

pub fn first_party_runtime_plugin_registrations_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> PluginSelectionResolutionReport<RuntimePluginRegistrationReport> {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let profile_manifest = profile.project_manifest();
    let manifest = manifest_with_mode_baseline(profile.target_mode, Some(&profile_manifest));
    first_party_runtime_plugin_registrations_for_manifest(profile.target_mode, &manifest)
}

pub fn first_party_runtime_plugin_registrations_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> PluginSelectionResolutionReport<RuntimePluginRegistrationReport> {
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
) -> PluginSelectionResolutionReport<RuntimePluginRegistrationReport> {
    first_party_runtime_catalog_for_manifest(target_mode, manifest).runtime_plugins
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
) -> PluginSelectionResolutionReport<RuntimePluginRegistrationReport> {
    resolve_plugin_selections(target_mode, manifest, |_| None)
}

#[cfg(test)]
#[path = "tests/first_party_runtime_plugins.rs"]
mod tests;
