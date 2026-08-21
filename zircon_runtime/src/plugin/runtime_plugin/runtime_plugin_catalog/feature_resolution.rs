use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginSelection;
use crate::plugin::PluginFeatureBundleManifest;

use super::derived_projection::RuntimePluginCatalogProjection;
use super::feature_blocking::block_unresolved_features;
use super::feature_capabilities::feature_capabilities_for_target;
use super::feature_report::RuntimePluginFeatureDependencyReport;
use super::feature_selection::PendingFeatureSelection;
use super::feature_status::feature_status;
use super::feature_status_record::FeatureStatus;

mod ordered_ready_set;

use ordered_ready_set::OrderedReadySet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct FeatureResolutionStats {
    pub(super) feature_status_evaluations: usize,
    pub(super) provider_registration_checks: usize,
    pub(super) provider_missing_blocks: usize,
    pub(super) dependency_edges_scanned: usize,
    pub(super) ready_queue_pushes: usize,
    pub(super) ready_queue_pops: usize,
}

/// Resolves each selected feature once and wakes only the rows affected by a new capability.
pub(super) fn resolve_pending_feature_dependencies<'a>(
    pending: Vec<PendingFeatureSelection<'a>>,
    projection: &RuntimePluginCatalogProjection,
    target: RuntimeTargetMode,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
    available_capabilities: &mut HashSet<String>,
    report: &mut RuntimePluginFeatureDependencyReport,
) -> FeatureResolutionStats {
    let definitions = projection.feature_definitions();
    let mut stats = FeatureResolutionStats::default();
    let mut current_ready = OrderedReadySet::new(pending.len());
    let mut next_ready = OrderedReadySet::new(pending.len());
    let mut waiting_by_capability = HashMap::<String, Vec<usize>>::new();
    let mut states =
        Vec::<Option<(PendingFeatureSelection<'a>, FeatureStatus)>>::with_capacity(pending.len());

    for (index, active) in pending.into_iter().enumerate() {
        let feature = definitions
            .definitions
            .get(&active.definition_key)
            .expect("unknown features removed before dependency resolution");
        let provider_registration_present =
            projection.has_concrete_feature_provider(&active.definition_key);
        stats.provider_registration_checks += 1;
        if !provider_registration_present {
            stats.provider_missing_blocks += 1;
            report.diagnostics.push(format!(
                "feature {} (provider {}) is blocked: concrete runtime feature provider registration is missing",
                feature.manifest.id, feature.provider_package_id
            ));
        }
        let status = feature_status(
            feature,
            active.active.feature,
            provider_registration_present,
            target,
            plugin_selections,
            enabled_plugins,
            available_capabilities,
        );
        stats.feature_status_evaluations += 1;
        stats.dependency_edges_scanned += feature.manifest.dependencies.len();
        if status.is_available() {
            states.push(None);
            publish_available_feature(
                index,
                active,
                &feature.manifest,
                target,
                available_capabilities,
                &waiting_by_capability,
                &mut states,
                &mut current_ready,
                &mut next_ready,
                true,
                report,
                &mut stats,
            );
        } else if status.is_immediately_blocked() {
            report
                .blocked_features
                .push(status.into_block(active.active.feature));
            states.push(None);
        } else {
            for capability in status.missing_capabilities() {
                waiting_by_capability
                    .entry(capability.clone())
                    .or_default()
                    .push(index);
            }
            states.push(Some((active, status)));
        }
    }

    while !current_ready.is_empty() || !next_ready.is_empty() {
        if current_ready.is_empty() {
            std::mem::swap(&mut current_ready, &mut next_ready);
        }
        let index = current_ready
            .pop_first()
            .expect("non-empty resolution pass should have a ready feature");
        stats.ready_queue_pops += 1;
        let Some((active, status)) = states[index].take() else {
            continue;
        };
        debug_assert!(status.is_available());
        let feature = definitions
            .definitions
            .get(&active.definition_key)
            .expect("ready feature definition should remain projected");
        publish_available_feature(
            index,
            active,
            &feature.manifest,
            target,
            available_capabilities,
            &waiting_by_capability,
            &mut states,
            &mut current_ready,
            &mut next_ready,
            false,
            report,
            &mut stats,
        );
    }

    block_unresolved_features(
        states.into_iter().flatten().collect(),
        projection,
        target,
        report,
    );
    stats
}

#[allow(clippy::too_many_arguments)]
fn publish_available_feature<'a>(
    index: usize,
    active: PendingFeatureSelection<'a>,
    manifest: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
    available_capabilities: &mut HashSet<String>,
    waiting_by_capability: &HashMap<String, Vec<usize>>,
    states: &mut [Option<(PendingFeatureSelection<'a>, FeatureStatus)>],
    current_ready: &mut OrderedReadySet,
    next_ready: &mut OrderedReadySet,
    initial_scan: bool,
    report: &mut RuntimePluginFeatureDependencyReport,
    stats: &mut FeatureResolutionStats,
) {
    report
        .available_features
        .push(active.active.feature.id.clone());
    for capability in feature_capabilities_for_target(manifest, target) {
        if !available_capabilities.insert(capability.to_string()) {
            continue;
        }
        let Some(waiting_indices) = waiting_by_capability.get(capability) else {
            continue;
        };
        for waiting_index in waiting_indices {
            let Some((_, waiting_status)) = states[*waiting_index].as_mut() else {
                continue;
            };
            if !waiting_status.resolve_missing_capability(capability)
                || !waiting_status.is_available()
            {
                continue;
            }
            let ready = if !initial_scan && *waiting_index > index {
                &mut *current_ready
            } else {
                &mut *next_ready
            };
            if ready.insert(*waiting_index) {
                stats.ready_queue_pushes += 1;
            }
        }
    }
}
