use std::collections::HashSet;

use crate::core::framework::platform::RuntimeTargetMode;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_selection::PendingFeatureSelection;
use super::super::feature_status_record::FeatureStatus;

pub(super) fn unresolved_feature_ids(
    pending: &[(PendingFeatureSelection<'_>, FeatureStatus)],
) -> HashSet<String> {
    pending
        .iter()
        .map(|(active, _)| active.definition_key.clone())
        .collect::<HashSet<_>>()
}

pub(super) fn mark_unresolved_feature_cycle(
    status: &mut FeatureStatus,
    projection: &RuntimePluginCatalogProjection,
    unresolved_feature_ids: &HashSet<String>,
    target: RuntimeTargetMode,
) {
    if status.is_waiting_for_feature_capability(projection, unresolved_feature_ids, target) {
        status.mark_cycle();
    }
}
