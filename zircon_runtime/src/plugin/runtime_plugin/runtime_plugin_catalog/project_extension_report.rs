mod enabled_packages;
mod feature_diagnostics;
mod feature_merge;
mod runtime_merge;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::{core::framework::project::ProjectPluginManifest, plugin::RuntimeExtensionRegistry};

use super::derived_projection::RuntimePluginCatalogProjection;
use super::extension_report::RuntimeExtensionCatalogReport;
use super::feature_registration_match::project_feature_provider_lookup;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};
use enabled_packages::enabled_plugin_ids_for_target;
use feature_diagnostics::append_feature_dependency_diagnostics;
use feature_merge::merge_available_feature_extensions;
use runtime_merge::merge_enabled_runtime_extensions;

pub(super) fn runtime_extension_report_for_project(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
    completed: &ProjectPluginManifest,
    target: RuntimeTargetMode,
    feature_report: &RuntimePluginFeatureDependencyReport,
) -> RuntimeExtensionCatalogReport {
    let enabled_plugins = enabled_plugin_ids_for_target(completed, target);
    let mut registry = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    if let Err(error) = merge_enabled_runtime_extensions(
        registrations,
        &enabled_plugins,
        &mut registry,
        &mut diagnostics,
        &mut fatal_diagnostics,
    ) {
        let diagnostic = format!("runtime plugin module descriptor ordering failed: {error}");
        diagnostics.push(diagnostic.clone());
        fatal_diagnostics.push(diagnostic);
        registry.finalize();
        return RuntimeExtensionCatalogReport {
            registry,
            diagnostics,
            fatal_diagnostics,
        };
    }
    append_feature_dependency_diagnostics(feature_report, &mut diagnostics, &mut fatal_diagnostics);
    let selected_feature_providers = project_feature_provider_lookup(completed);
    merge_available_feature_extensions(
        feature_registrations,
        projection,
        &selected_feature_providers,
        feature_report,
        &mut registry,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    registry.finalize();
    RuntimeExtensionCatalogReport {
        registry,
        diagnostics,
        fatal_diagnostics,
    }
}
