use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginSelection;

mod availability;

use super::feature_blocking::block_unresolved_features;
use super::feature_definitions::FeatureDefinitionMap;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::feature_selection::PendingFeatureSelection;
use availability::resolve_available_features;

/// Expands feature-provided capabilities to a fixed point before projecting unresolved waits as blocks.
pub(super) fn resolve_pending_feature_dependencies<'a>(
    mut pending: Vec<PendingFeatureSelection<'a>>,
    feature_definitions: &FeatureDefinitionMap,
    target: RuntimeTargetMode,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &mut HashSet<String>,
    report: &mut RuntimePluginFeatureDependencyReport,
) {
    resolve_available_features(
        &mut pending,
        feature_definitions,
        target,
        plugin_selections,
        enabled_plugins,
        available_capabilities,
        report,
    );
    block_unresolved_features(
        pending,
        feature_definitions,
        target,
        plugin_selections,
        enabled_plugins,
        available_capabilities,
        report,
    );
}
