mod feature_diagnostics;
mod feature_merge;
mod runtime_merge;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::RuntimeExtensionRegistry;

use super::extension_report::RuntimeExtensionCatalogReport;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::project::CompiledRuntimePluginSelection;
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};
use feature_diagnostics::append_feature_dependency_diagnostics;
use feature_merge::merge_selected_feature_extensions;
use runtime_merge::merge_selected_runtime_extensions;

// 基础选择发生致命错误时直接返回空的已冻结扩展表，避免部分注册泄露给调用方。
pub(super) fn runtime_extension_report_for_project(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    target: RuntimeTargetMode,
    feature_report: &RuntimePluginFeatureDependencyReport,
    selection: &CompiledRuntimePluginSelection,
) -> RuntimeExtensionCatalogReport {
    let mut registry = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    if let Some(diagnostic) = selection.fatal_diagnostic() {
        diagnostics.push(diagnostic.to_string());
        fatal_diagnostics.push(diagnostic.to_string());
        registry.finalize();
        return RuntimeExtensionCatalogReport {
            registry,
            diagnostics,
            fatal_diagnostics,
        };
    }
    merge_selected_runtime_extensions(
        registrations,
        selection.ordered_plugin_registration_indices(),
        target,
        &mut registry,
        &mut diagnostics,
        &mut fatal_diagnostics,
    );
    append_feature_dependency_diagnostics(feature_report, &mut diagnostics, &mut fatal_diagnostics);
    merge_selected_feature_extensions(
        feature_registrations,
        selection.feature_registration_indices(),
        target,
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
