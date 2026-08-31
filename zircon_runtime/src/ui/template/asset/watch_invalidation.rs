use std::collections::HashSet;

use crate::asset::watch::{AssetChange, AssetChangeKind};

use super::dependency_index::UiAssetDependencyIndex;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetWatchInvalidationReport {
    pub changed_assets: Vec<String>,
    pub rebuild_targets: Vec<String>,
    pub removed_assets: Vec<String>,
}

impl UiAssetDependencyIndex {
    pub fn apply_watch_changes(
        &mut self,
        changes: &[AssetChange],
    ) -> UiAssetWatchInvalidationReport {
        let mut report = UiAssetWatchInvalidationReport::default();
        let mut changed_seen = HashSet::new();
        let mut rebuild_seen = HashSet::new();
        let mut removed_seen = HashSet::new();

        for change in changes {
            if let Some(previous_uri) = change.previous_uri.as_ref() {
                let previous = previous_uri.to_string();
                if !removed_seen.contains(previous.as_str()) {
                    removed_seen.insert(previous.clone());
                    report.removed_assets.push(previous.clone());
                }
                for target in self.cascade_invalidation_targets(&previous) {
                    if !rebuild_seen.contains(target.as_str()) {
                        rebuild_seen.insert(target.clone());
                        report.rebuild_targets.push(target);
                    }
                }
                self.remove(&previous);
            }

            let changed = change.uri.to_string();
            if !changed_seen.contains(changed.as_str()) {
                changed_seen.insert(changed.clone());
                report.changed_assets.push(changed.clone());
            }

            for target in self.cascade_invalidation_targets(&changed) {
                if !rebuild_seen.contains(target.as_str()) {
                    rebuild_seen.insert(target.clone());
                    report.rebuild_targets.push(target);
                }
            }

            match &change.kind {
                AssetChangeKind::Removed => {
                    if !removed_seen.contains(changed.as_str()) {
                        removed_seen.insert(changed.clone());
                        report.removed_assets.push(changed.clone());
                    }
                    self.remove(&changed);
                }
                AssetChangeKind::Added | AssetChangeKind::Modified | AssetChangeKind::Renamed => {}
            }
        }

        report
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::asset::AssetUri;

    use super::*;

    const ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_ASSET_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn nearest_rank(samples: &mut [Duration], percentile: usize) -> Duration {
        samples.sort_unstable();
        let rank = samples.len().saturating_mul(percentile).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    fn asset_ids() -> Vec<String> {
        (0..ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "res://ui/watch/{:05}.zui",
                    (index * 4_099) % UNIQUE_ASSET_COUNT
                )
            })
            .collect()
    }

    fn ordered_admission(asset_ids: &[String]) -> Vec<String> {
        let mut seen = BTreeSet::new();
        let mut admitted = Vec::with_capacity(UNIQUE_ASSET_COUNT);
        for asset_id in asset_ids {
            if seen.insert(asset_id.clone()) {
                admitted.push(asset_id.clone());
            }
        }
        admitted
    }

    fn hash_admission(asset_ids: &[String]) -> Vec<String> {
        let mut seen = HashSet::with_capacity(UNIQUE_ASSET_COUNT);
        let mut admitted = Vec::with_capacity(UNIQUE_ASSET_COUNT);
        for asset_id in asset_ids {
            if !seen.contains(asset_id.as_str()) {
                seen.insert(asset_id.clone());
                admitted.push(asset_id.clone());
            }
        }
        admitted
    }

    fn uri(value: &str) -> AssetUri {
        AssetUri::parse(value).unwrap()
    }

    #[test]
    fn runtime74_batch_watch_hash_admission_preserves_first_seen_order() {
        let mut index = UiAssetDependencyIndex::new();
        let report = index.apply_watch_changes(&[
            AssetChange::new(AssetChangeKind::Modified, uri("res://ui/views/b.zui"), None),
            AssetChange::new(AssetChangeKind::Modified, uri("res://ui/views/a.zui"), None),
            AssetChange::new(AssetChangeKind::Removed, uri("res://ui/views/b.zui"), None),
        ]);

        assert_eq!(
            report.changed_assets,
            vec![
                "res://ui/views/b.zui".to_string(),
                "res://ui/views/a.zui".to_string(),
            ]
        );
        assert_eq!(
            report.removed_assets,
            vec!["res://ui/views/b.zui".to_string()]
        );
    }

    #[test]
    fn runtime74_batch_watch_invalidation_uses_borrowed_hash_admission() {
        let source = include_str!("watch_invalidation.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("changed_seen.contains(changed.as_str())"));
        assert!(production.contains("rebuild_seen.contains(target.as_str())"));
        assert!(production.contains("removed_seen.contains(previous.as_str())"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn runtime74_batch_watch_hash_admission_performance_evidence() {
        let asset_ids = asset_ids();
        assert_eq!(ordered_admission(&asset_ids), hash_admission(&asset_ids));

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_admission(black_box(&asset_ids)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_admission(black_box(&asset_ids)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_admission(black_box(&asset_ids)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_admission(black_box(&asset_ids)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p50 = nearest_rank(&mut ordered_samples.clone(), 50);
        let ordered_p95 = nearest_rank(&mut ordered_samples, 95);
        let hash_p50 = nearest_rank(&mut hash_samples.clone(), 50);
        let hash_p95 = nearest_rank(&mut hash_samples, 95);
        println!(
            "RUNTIME74_WATCH_INVALIDATION_HASH_ADMISSION_BENCH_V1 admissions={ADMISSION_COUNT} \
             unique_assets={UNIQUE_ASSET_COUNT} sample_pairs={SAMPLE_COUNT} \
             pair_order=alternating_ordered_even ordered_first_pairs=9 hash_first_pairs=8 \
             ordered_owned_allocations={} hash_owned_allocations={} \
             ordered_p50_ns={} ordered_p95_ns={} hash_p50_ns={} hash_p95_ns={}",
            ADMISSION_COUNT + UNIQUE_ASSET_COUNT,
            UNIQUE_ASSET_COUNT * 2,
            ordered_p50.as_nanos(),
            ordered_p95.as_nanos(),
            hash_p50.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-admission P95 {:?} exceeded 60% of ordered-admission P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
