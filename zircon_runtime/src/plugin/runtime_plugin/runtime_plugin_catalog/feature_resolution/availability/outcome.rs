use std::collections::HashSet;

use crate::builtin::RuntimeTargetMode;

use super::super::super::feature_capabilities::feature_capabilities_for_target;
use super::super::super::feature_definitions::FeatureDefinition;
use super::super::super::feature_report::RuntimePluginFeatureDependencyReport;
use super::super::super::feature_selection::PendingFeatureSelection;
use super::super::super::feature_status_record::FeatureStatus;

pub(super) enum FeatureResolutionOutcome {
    Available,
    Blocked,
    Waiting,
}

pub(super) fn apply_feature_resolution_status<'a>(
    pending: &mut Vec<PendingFeatureSelection<'a>>,
    index: usize,
    feature: &FeatureDefinition,
    status: FeatureStatus,
    target: RuntimeTargetMode,
    available_capabilities: &mut HashSet<String>,
    report: &mut RuntimePluginFeatureDependencyReport,
) -> FeatureResolutionOutcome {
    if status.is_available() {
        let active = pending.remove(index);
        report
            .available_features
            .push(active.active.feature.id.clone());
        available_capabilities.extend(feature_capabilities_for_target(&feature.manifest, target));
        FeatureResolutionOutcome::Available
    } else if status.is_immediately_blocked() {
        let active = pending.remove(index);
        report
            .blocked_features
            .push(status.into_block(active.active.feature));
        FeatureResolutionOutcome::Blocked
    } else {
        FeatureResolutionOutcome::Waiting
    }
}
