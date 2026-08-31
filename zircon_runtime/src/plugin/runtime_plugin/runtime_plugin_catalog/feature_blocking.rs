use crate::core::framework::platform::RuntimeTargetMode;

mod cycle;

use super::derived_projection::RuntimePluginCatalogProjection;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::feature_selection::PendingFeatureSelection;
use super::feature_status_record::FeatureStatus;
use cycle::{mark_unresolved_feature_cycle, unresolved_feature_ids};

pub(super) fn block_unresolved_features(
    mut pending: Vec<(PendingFeatureSelection<'_>, FeatureStatus)>,
    projection: &RuntimePluginCatalogProjection,
    target: RuntimeTargetMode,
    report: &mut RuntimePluginFeatureDependencyReport,
) {
    let unresolved_feature_ids = unresolved_feature_ids(&mut pending);
    for (active, mut status) in pending {
        debug_assert!(!status.is_immediately_blocked());
        mark_unresolved_feature_cycle(&mut status, projection, &unresolved_feature_ids, target);
        report
            .blocked_features
            .push(status.into_block(active.active.feature));
    }
}
