use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginSelection;

mod cycle;

use super::feature_definitions::FeatureDefinitionMap;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::feature_selection::PendingFeatureSelection;
use super::feature_status::feature_status;
use cycle::{mark_unresolved_feature_cycle, unresolved_feature_ids};

pub(super) fn block_unresolved_features(
    pending: Vec<PendingFeatureSelection<'_>>,
    feature_definitions: &FeatureDefinitionMap,
    target: RuntimeTargetMode,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &HashSet<String>,
    report: &mut RuntimePluginFeatureDependencyReport,
) {
    let unresolved_feature_ids = unresolved_feature_ids(&pending);
    for active in pending {
        let feature = feature_definitions
            .definitions
            .get(&active.definition_key)
            .expect("unknown features removed before dependency resolution");
        let mut status = feature_status(
            feature,
            active.active.feature,
            target,
            plugin_selections,
            enabled_plugins,
            available_capabilities,
        );
        mark_unresolved_feature_cycle(
            &mut status,
            feature_definitions,
            &unresolved_feature_ids,
            target,
        );
        report
            .blocked_features
            .push(status.into_block(active.active.feature));
    }
}
