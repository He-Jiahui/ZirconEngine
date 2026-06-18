use std::collections::HashSet;

use crate::builtin::RuntimeTargetMode;

use super::super::feature_definitions::FeatureDefinitionMap;
use super::super::feature_selection::PendingFeatureSelection;
use super::super::feature_status_record::FeatureStatus;

pub(super) fn unresolved_feature_ids(pending: &[PendingFeatureSelection<'_>]) -> HashSet<String> {
    pending
        .iter()
        .map(|active| active.definition_key.clone())
        .collect::<HashSet<_>>()
}

pub(super) fn mark_unresolved_feature_cycle(
    status: &mut FeatureStatus,
    feature_definitions: &FeatureDefinitionMap,
    unresolved_feature_ids: &HashSet<String>,
    target: RuntimeTargetMode,
) {
    if status.is_waiting_for_feature_capability(
        &feature_definitions.definitions,
        unresolved_feature_ids,
        target,
    ) {
        status.mark_cycle();
    }
}
