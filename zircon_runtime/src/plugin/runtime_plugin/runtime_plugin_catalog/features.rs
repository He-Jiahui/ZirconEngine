mod context;

use crate::{plugin::ProjectPluginManifest, RuntimeTargetMode};

use super::feature_definition_collection::feature_definition_map;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::feature_resolution::resolve_pending_feature_dependencies;
use super::feature_selection::feature_selection_partition;
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};
use context::feature_dependency_context;

pub(super) fn feature_dependency_report(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    completed: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> RuntimePluginFeatureDependencyReport {
    let feature_definitions = feature_definition_map(registrations, feature_registrations);
    let mut context =
        feature_dependency_context(registrations, completed, target, &feature_definitions);
    let selection_partition = feature_selection_partition(completed, &feature_definitions);
    context
        .report
        .blocked_features
        .extend(selection_partition.unknown_feature_blocks);
    resolve_pending_feature_dependencies(
        selection_partition.pending,
        &feature_definitions,
        target,
        &context.plugin_selections,
        &context.enabled_plugins,
        &mut context.available_capabilities,
        &mut context.report,
    );

    context.report
}
