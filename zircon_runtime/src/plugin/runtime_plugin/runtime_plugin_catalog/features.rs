mod context;

use std::collections::HashSet;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

use super::derived_projection::RuntimePluginCatalogProjection;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::feature_resolution::{resolve_pending_feature_dependencies, FeatureResolutionStats};
use super::feature_selection::feature_selection_partition;
#[cfg(test)]
use context::feature_dependency_context;
use context::{feature_dependency_context_for_effective_base, FeatureDependencyContext};

#[cfg(test)]
pub(super) fn feature_dependency_report_with_stats(
    projection: &RuntimePluginCatalogProjection,
    completed: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> (RuntimePluginFeatureDependencyReport, FeatureResolutionStats) {
    let feature_definitions = projection.feature_definitions();
    let context = feature_dependency_context(projection, completed, target, feature_definitions);
    resolve_feature_dependency_report(projection, completed, target, context)
}

pub(super) fn feature_dependency_report_for_effective_base(
    projection: &RuntimePluginCatalogProjection,
    completed: &ProjectPluginManifest,
    target: RuntimeTargetMode,
    enabled_plugins: HashSet<String>,
    available_capabilities: HashSet<String>,
) -> RuntimePluginFeatureDependencyReport {
    let feature_definitions = projection.feature_definitions();
    let context = feature_dependency_context_for_effective_base(
        completed,
        feature_definitions,
        enabled_plugins,
        available_capabilities,
    );
    resolve_feature_dependency_report(projection, completed, target, context).0
}

fn resolve_feature_dependency_report<'a>(
    projection: &RuntimePluginCatalogProjection,
    completed: &'a ProjectPluginManifest,
    target: RuntimeTargetMode,
    mut context: FeatureDependencyContext<'a>,
) -> (RuntimePluginFeatureDependencyReport, FeatureResolutionStats) {
    let feature_definitions = projection.feature_definitions();
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
