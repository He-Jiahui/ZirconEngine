use std::collections::{HashMap, HashSet};

use crate::builtin::RuntimeTargetMode;
use crate::plugin::ProjectPluginSelection;

use super::super::feature_definitions::FeatureDefinitionMap;
use super::super::feature_report::RuntimePluginFeatureDependencyReport;
use super::super::feature_selection::PendingFeatureSelection;
use super::super::feature_status::feature_status;

mod outcome;

use outcome::{apply_feature_resolution_status, FeatureResolutionOutcome};

pub(super) fn resolve_available_features<'a>(
    pending: &mut Vec<PendingFeatureSelection<'a>>,
    feature_definitions: &FeatureDefinitionMap,
    target: RuntimeTargetMode,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &mut HashSet<String>,
    report: &mut RuntimePluginFeatureDependencyReport,
) {
    let mut made_progress = true;
    while made_progress && !pending.is_empty() {
        made_progress = false;
        let mut index = 0;
        while index < pending.len() {
            let definition_key = pending[index].definition_key.clone();
            let feature = feature_definitions
                .definitions
                .get(&definition_key)
                .expect("unknown features removed before dependency resolution");
            let status = feature_status(
                feature,
                pending[index].active.feature,
                target,
                plugin_selections,
                enabled_plugins,
                available_capabilities,
            );
            match apply_feature_resolution_status(
                pending,
                index,
                feature,
                status,
                target,
                available_capabilities,
                report,
            ) {
                FeatureResolutionOutcome::Available => {
                    made_progress = true;
                }
                FeatureResolutionOutcome::Blocked => {}
                FeatureResolutionOutcome::Waiting => {
                    index += 1;
                }
            }
        }
    }
}
