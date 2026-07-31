mod context;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

use super::derived_projection::RuntimePluginCatalogProjection;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::feature_resolution::{resolve_pending_feature_dependencies, FeatureResolutionStats};
use super::feature_selection::feature_selection_partition;
use context::feature_dependency_context;

pub(super) fn feature_dependency_report(
    projection: &RuntimePluginCatalogProjection,
    completed: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> RuntimePluginFeatureDependencyReport {
    feature_dependency_report_with_stats(projection, completed, target).0
}

pub(super) fn feature_dependency_report_with_stats(
    projection: &RuntimePluginCatalogProjection,
    completed: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> (RuntimePluginFeatureDependencyReport, FeatureResolutionStats) {
    let feature_definitions = projection.feature_definitions();
    let mut context =
        feature_dependency_context(projection, completed, target, feature_definitions);
    let selection_partition = feature_selection_partition(completed, &feature_definitions);
    context
        .report
        .blocked_features
        .extend(selection_partition.unknown_feature_blocks);
    let stats = resolve_pending_feature_dependencies(
        selection_partition.pending,
        projection,
        target,
        &context.plugin_selections,
        &context.enabled_plugins,
        &mut context.available_capabilities,
        &mut context.report,
    );

    (context.report, stats)
}
