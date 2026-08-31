//! Atomic lifecycle cleanup and activation for plugin instance replacement.

use std::collections::HashSet;

use super::super::catalog::EditorPluginCatalog;
use super::super::sdk::lifecycle::{EditorPluginLifecycleEvent, EditorPluginLifecycleStage};
use super::discovery::EditorPluginDiscoveryError;
use super::snapshot::{EditorPluginManagerEntry, EditorPluginManagerSnapshot};
use super::state::EditorPluginState;

pub(super) fn reset_replaced_active_entries(
    previous_catalog: &EditorPluginCatalog,
    candidate_catalog: &EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
) {
    for entry in entries.iter_mut().filter(|entry| {
        matches!(
            entry.state,
            EditorPluginState::Faulted | EditorPluginState::Active | EditorPluginState::Revoking
        )
    }) {
        if !candidate_catalog.is_package_faulted(entry.package_id())
            && !candidate_catalog.has_same_lifecycle_plugin(previous_catalog, entry.package_id())
        {
            entry.state = EditorPluginState::Validated;
        }
    }
}

pub(super) fn replaced_live_package_ids(
    previous: &EditorPluginManagerSnapshot,
    previous_catalog: &EditorPluginCatalog,
    candidate_catalog: &EditorPluginCatalog,
) -> HashSet<String> {
    previous
        .entries()
        .iter()
        .filter(|entry| instance_requires_retirement(previous_catalog, entry))
        .filter(|entry| {
            !candidate_catalog.has_same_lifecycle_plugin(previous_catalog, entry.package_id())
        })
        .map(|entry| entry.package_id.clone())
        .collect()
}

fn instance_requires_retirement(
    catalog: &EditorPluginCatalog,
    entry: &EditorPluginManagerEntry,
) -> bool {
    entry.state == EditorPluginState::Active
        || entry.state == EditorPluginState::Revoking
        || (entry.state == EditorPluginState::Faulted
            && catalog.lifecycle_stage_succeeded(
                entry.package_id(),
                &EditorPluginLifecycleStage::Enabled,
            ))
}

pub(super) fn retire_replaced_active_entries(
    catalog: &mut EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
    replaced_live_package_ids: &HashSet<String>,
) -> Result<(), EditorPluginDiscoveryError> {
    let retirement_ids = entries
        .iter()
        .filter(|entry| {
            replaced_live_package_ids.contains(entry.package_id())
                && instance_requires_retirement(catalog, entry)
        })
        .map(|entry| entry.package_id.clone())
        .collect::<HashSet<_>>();
    for entry in entries
        .iter_mut()
        .filter(|entry| retirement_ids.contains(entry.package_id()))
    {
        entry.state = EditorPluginState::Revoking;
        for stage in [
            EditorPluginLifecycleStage::Disabled,
            EditorPluginLifecycleStage::Unloaded,
        ] {
            if catalog.lifecycle_stage_succeeded(entry.package_id(), &stage) {
                continue;
            }
            let report = catalog.record_lifecycle_event(
                entry.package_id(),
                EditorPluginLifecycleEvent::new(stage.clone()),
            );
            if !report.is_success() {
                entry.state = EditorPluginState::Faulted;
                return Err(EditorPluginDiscoveryError::LifecycleCleanupFailed {
                    package_id: entry.package_id.clone(),
                    stage,
                });
            }
        }
    }
    Ok(())
}

pub(super) fn dispatch_hot_reloaded_replacements(
    catalog: &mut EditorPluginCatalog,
    entries: &mut [EditorPluginManagerEntry],
    replaced_live_package_ids: &HashSet<String>,
) {
    for entry in entries.iter_mut().filter(|entry| {
        replaced_live_package_ids.contains(entry.package_id())
            && entry.state == EditorPluginState::Active
    }) {
        let report = catalog.record_lifecycle_event(
            entry.package_id(),
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::HotReloaded),
        );
        if !report.is_success() {
            entry.state = EditorPluginState::Faulted;
        }
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::core::plugin::EditorPluginDescriptor;

    use super::super::{EditorPluginLoadingPhase, EditorPluginManager};
    use super::*;

    const PACKAGE_COUNT: usize = 32_768;
    const MEMBERSHIP_PASSES: usize = 3;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn package_ids() -> Vec<String> {
        (0..PACKAGE_COUNT)
            .map(|index| format!("plugin.replacement.{:05}", (index * 16_381) % PACKAGE_COUNT))
            .collect()
    }

    fn legacy_membership_checksum(package_ids: &[String]) -> usize {
        let replaced = package_ids.iter().cloned().collect::<BTreeSet<_>>();
        let retirement = package_ids
            .iter()
            .filter(|package_id| replaced.contains(package_id.as_str()))
            .cloned()
            .collect::<BTreeSet<_>>();
        let retired = package_ids
            .iter()
            .filter(|package_id| retirement.contains(package_id.as_str()))
            .count();
        let dispatched = package_ids
            .iter()
            .filter(|package_id| replaced.contains(package_id.as_str()))
            .count();
        retirement.len() + retired + dispatched
    }

    fn optimized_membership_checksum(package_ids: &[String]) -> usize {
        let replaced = package_ids.iter().cloned().collect::<HashSet<_>>();
        let retirement = package_ids
            .iter()
            .filter(|package_id| replaced.contains(package_id.as_str()))
            .cloned()
            .collect::<HashSet<_>>();
        let retired = package_ids
            .iter()
            .filter(|package_id| retirement.contains(package_id.as_str()))
            .count();
        let dispatched = package_ids
            .iter()
            .filter(|package_id| replaced.contains(package_id.as_str()))
            .count();
        retirement.len() + retired + dispatched
    }

    #[test]
    fn optimization_batch_20260826p_editor06_hash_membership_preserves_replacement_state_order() {
        let descriptors = [
            EditorPluginDescriptor::new("plugin.z", "Plugin Z", "plugin_z"),
            EditorPluginDescriptor::new("plugin.a", "Plugin A", "plugin_a"),
        ];
        let manager = EditorPluginManager::from_descriptors(descriptors, []).unwrap();
        manager
            .advance_loading_phase(EditorPluginLoadingPhase::Default)
            .unwrap();

        let replacements = [
            EditorPluginDescriptor::new("plugin.z", "Plugin Z2", "plugin_z2"),
            EditorPluginDescriptor::new("plugin.a", "Plugin A2", "plugin_a2"),
        ];
        manager
            .publish_catalog(EditorPluginCatalog::from_descriptors(replacements, []))
            .unwrap();
        let snapshot = manager.state_snapshot();

        assert_eq!(
            snapshot
                .entries()
                .iter()
                .map(|entry| entry.package_id())
                .collect::<Vec<_>>(),
            vec!["plugin.a", "plugin.z"]
        );
        assert!(snapshot
            .entries()
            .iter()
            .all(|entry| entry.state() == EditorPluginState::Active));
    }

    #[test]
    fn optimization_batch_20260826p_editor06_replacement_uses_hash_membership_sets() {
        let source = include_str!("lifecycle_replacement.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert_eq!(production.matches("HashSet<String>").count(), 3);
        assert!(production.contains("collect::<HashSet<_>>()"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826p_editor06_plugin_replacement_hash_membership_performance_evidence(
    ) {
        let package_ids = package_ids();
        let expected = PACKAGE_COUNT * MEMBERSHIP_PASSES;
        assert_eq!(legacy_membership_checksum(&package_ids), expected);
        assert_eq!(optimized_membership_checksum(&package_ids), expected);

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_membership_checksum(black_box(&package_ids)));
                legacy_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(optimized_membership_checksum(black_box(&package_ids)));
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_membership_checksum(black_box(&package_ids)));
                optimized_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(legacy_membership_checksum(black_box(&package_ids)));
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "EDITOR06_PLUGIN_REPLACEMENT_HASH_MEMBERSHIP_BENCH_V1 packages={PACKAGE_COUNT} \
             membership_passes={MEMBERSHIP_PASSES} ordered_admissions={} \
             hash_admissions={} membership_probes={} legacy_p95_ns={} \
             optimized_p95_ns={}",
            PACKAGE_COUNT * 2,
            PACKAGE_COUNT * 2,
            PACKAGE_COUNT * MEMBERSHIP_PASSES,
            legacy_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
            "hash-membership P95 {:?} exceeded 60% of ordered-membership P95 {:?}",
            optimized_p95,
            legacy_p95,
        );
    }
}
