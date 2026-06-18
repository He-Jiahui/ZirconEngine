mod enabled_packages;
mod feature_diagnostics;
mod feature_merge;
mod runtime_merge;

use crate::builtin::RuntimeTargetMode;
use crate::{plugin::ProjectPluginManifest, plugin::RuntimeExtensionRegistry};

use super::extension_report::RuntimeExtensionCatalogReport;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};
use enabled_packages::enabled_plugin_ids_for_target;
use feature_diagnostics::append_feature_dependency_diagnostics;
use feature_merge::merge_available_feature_extensions;
use runtime_merge::merge_enabled_runtime_extensions;

pub(super) fn runtime_extension_report_for_project(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    completed: &ProjectPluginManifest,
    target: RuntimeTargetMode,
    feature_report: &RuntimePluginFeatureDependencyReport,
) -> RuntimeExtensionCatalogReport {
    let enabled_plugins = enabled_plugin_ids_for_target(completed, target);
    let mut registry = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    merge_enabled_runtime_extensions(
        registrations,
        &enabled_plugins,
        &mut registry,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    append_feature_dependency_diagnostics(feature_report, &mut diagnostics, &mut fatal_diagnostics);
    merge_available_feature_extensions(
        feature_registrations,
        completed,
        feature_report,
        &mut registry,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    RuntimeExtensionCatalogReport {
        registry,
        diagnostics,
        fatal_diagnostics,
    }
}
