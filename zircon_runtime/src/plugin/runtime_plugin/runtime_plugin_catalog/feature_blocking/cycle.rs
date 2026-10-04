use std::collections::HashSet;

use crate::core::framework::platform::RuntimeTargetMode;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_selection::PendingFeatureSelection;
use super::super::feature_status_record::FeatureStatus;

// 待解析键在此转移进循环判定集合，之后不再由 pending 记录读取。
pub(super) fn unresolved_feature_ids(
    pending: &mut [(PendingFeatureSelection<'_>, FeatureStatus)],
) -> HashSet<String> {
    pending
        .iter_mut()
        .map(|(active, _)| std::mem::take(&mut active.definition_key))
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

#[cfg(test)]
#[path = "tests/cycle_performance_tests.rs"]
mod performance_tests;
